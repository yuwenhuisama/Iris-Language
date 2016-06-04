#ifndef _H_IRISCLOSUREBLOCK_
#define _H_IRISCLOSUREBLOCK_

#include "IrisUnil/IrisValue.h"
#include "IrisUnil/IrisList.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"
#include "IrisUnil/IrisCodeSegment.h"
#include "IrisThread/IrisWLLock.h"

#include "IrisInterfaces/IIrisClosureBlock.h"
#include "IrisInterfaces/IIrisContextEnvironment.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"

#include "IrisUnil/IrisInternString.h"

#include <unordered_map>
#include <vector>
#include <list>
using namespace std;

class IIrisValues;
class IIrisContextEnvironment;

class IrisObject;
class IrisIdentifier;
class IrisStatement;
class IrisContextEnvironment;
#ifdef IR_USE_MEM_POOL
class IrisClosureBlock : public IIrisClosureBlock, public IrisObjectMemoryPoolInterface<IrisClosureBlock, POOLID_IrisClosureBlock>
#else
class IrisClosureBlock : public IIrisClosureBlock
#endif
{
private:
	unsigned int m_nIndex = 0;
	IrisContextEnvironment* m_pCurContextEnvironment = nullptr;
	IrisContextEnvironment* m_pUpperContextEnvironment = nullptr;
	IrisObject* m_pNativeObject = nullptr;

	list<IrisInternString> m_lsParameters;
	IrisCodeSegment m_icsCodes;

private:
	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _VariableMap;
	typedef pair<IrisInternString, IrisValue> _VariablePair;

private:
	IrisContextEnvironment* _CreateNewContextEnvironment();

private:
	_VariableMap m_mpOtherVariables;
	IrisWLLock m_iwlVariableLock;

public:

	inline IIrisContextEnvironment* GetCurrentContextEnvironment() {
		return m_pCurContextEnvironment;
	}

	const IrisValue& GetLocalVariable(const IrisInternString& strVariableName, bool& bResult);
	const IrisValue& GetInstanceVariable(const IrisInternString& strVariableName, bool& bResult);
	const IrisValue& GetClassVariable(const IrisInternString& strVariableName, bool& bResult);
	const IrisValue& GetConstance(const IrisInternString& strConstanceName, bool& bResult);

	void AddLocalVariable(const IrisInternString& strVariableName, const IrisValue& ivValue);
	
	void AddOtherVariable(const IrisInternString& strVariableName, const IrisValue& ivValue);

	void Mark();

	IrisValue Excute(IIrisValues* pValues);

	IrisClosureBlock(IrisContextEnvironment* pUpperContexEnvironment, list<IrisInternString>& lsParameters, unsigned int nStartPointer, unsigned int nEndPointer, vector<IR_WORD>& lsCodes, int nBelongingFileIndex, unsigned int nIndex);
	~IrisClosureBlock();

	friend class IrisClosureBlockBase;
	friend class IrisExpression;
};

#endif
#ifndef _H_IRISCLOSUREBLOCK_
#define _H_IRISCLOSUREBLOCK_

#include "../../IrisCompileConfigure.h"

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
#if IR_USE_MEM_POOL
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

#if IR_USE_STL_STRING
	list<string> m_lsParameters;
#else
	list<IrisInternString> m_lsParameters;
#endif // IR_USE_STL_STRING

	IrisCodeSegment m_icsCodes;

private:
#if IR_USE_STL_STRING
	typedef unordered_map<string, IrisValue> _VariableMap;
	typedef pair<string, IrisValue> _VariablePair;
#else
	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _VariableMap;
	typedef pair<IrisInternString, IrisValue> _VariablePair;
#endif // IR_USE_STL_STRING

private:
	IrisContextEnvironment* _CreateNewContextEnvironment();

private:
	_VariableMap m_mpOtherVariables;
	IrisWLLock m_iwlVariableLock;

public:

	inline IIrisContextEnvironment* GetCurrentContextEnvironment() {
		return m_pCurContextEnvironment;
	}

#if IR_USE_STL_STRING
	const IrisValue& GetLocalVariable(const string& strVariableName, bool& bResult);
	const IrisValue& GetInstanceVariable(const string& strVariableName, bool& bResult);
	const IrisValue& GetClassVariable(const string& strVariableName, bool& bResult);
	const IrisValue& GetConstance(const string& strConstanceName, bool& bResult);
	void AddLocalVariable(const string& strVariableName, const IrisValue& ivValue);
	void AddOtherVariable(const string& strVariableName, const IrisValue& ivValue);
#else
	const IrisValue& GetLocalVariable(const IrisInternString& strVariableName, bool& bResult);
	const IrisValue& GetInstanceVariable(const IrisInternString& strVariableName, bool& bResult);
	const IrisValue& GetClassVariable(const IrisInternString& strVariableName, bool& bResult);
	const IrisValue& GetConstance(const IrisInternString& strConstanceName, bool& bResult);
	void AddLocalVariable(const IrisInternString& strVariableName, const IrisValue& ivValue);
	void AddOtherVariable(const IrisInternString& strVariableName, const IrisValue& ivValue);
#endif // IR_USE_STL_STRING



	void Mark();

	IrisValue Excute(IIrisValues* pValues);

#if IR_USE_STL_STRING
	IrisClosureBlock(IrisContextEnvironment* pUpperContexEnvironment, list<string>& lsParameters, unsigned int nStartPointer, unsigned int nEndPointer, vector<IR_WORD>& lsCodes, int nBelongingFileIndex, unsigned int nIndex);
#else
	IrisClosureBlock(IrisContextEnvironment* pUpperContexEnvironment, list<IrisInternString>& lsParameters, unsigned int nStartPointer, unsigned int nEndPointer, vector<IR_WORD>& lsCodes, int nBelongingFileIndex, unsigned int nIndex);
#endif // IR_USE_STL_STRING

	~IrisClosureBlock();

	friend class IrisClosureBlockBase;
	friend class IrisExpression;
};

#endif
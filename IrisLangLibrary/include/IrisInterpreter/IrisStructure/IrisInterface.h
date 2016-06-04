#ifndef _H_IRISINTERFACE_
#define _H_IRISINTERFACE_

#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"
#include "IrisThread/IrisWLLock.h"

#include "IrisUnil/IrisInternString.h"

#include <string>
#include <unordered_map>
#include <unordered_set>
using namespace std;

class IrisInterface;
class IrisModule;
class IrisObject;

class IIrisInterface;
class IIrisObject;

#ifdef IR_USE_MEM_POOL
class IrisInterface : public IrisObjectMemoryPoolInterface<IrisInterface, POOLID_IrisInterface> 
#else
class IrisInterface
#endif
{
public:
	struct InterfaceFunctionDeclare {
		IrisInternString m_strInterfaceName = "";
		int m_nParameterAmount = 0;
		bool m_bHaveVariableParameter = false;

		InterfaceFunctionDeclare(const IrisInternString& strInterfaceName, int nParameterAmount, bool b_HaveVariableParameter) : m_strInterfaceName(strInterfaceName), m_nParameterAmount(nParameterAmount), m_bHaveVariableParameter(b_HaveVariableParameter){ }
		InterfaceFunctionDeclare() { }
	};

private:
	typedef unordered_map<IrisInternString, InterfaceFunctionDeclare, IrisInternString::IrisInerStringHash> _InterfaceFunctionDeclareMap;
	typedef pair<IrisInternString, InterfaceFunctionDeclare> _InterfaceFunctionDeclarePair;
	typedef unordered_set<IrisInterface*> _InterfaceSet;
	typedef pair<IrisInternString, IrisInterface*> _InterfacePair;

private:
	IrisInternString m_strInterfaceName = "";
	IrisModule* m_pUpperModule = nullptr;
	_InterfaceFunctionDeclareMap m_mpFunctionDeclareMap;
	_InterfaceSet m_mpInterfaces;

	IrisWLLock m_iwlDecAddingLock;
	IrisWLLock m_iwlInfAddingLock;

private:
	IIrisObject* m_pInterfaceObject = nullptr;
	IIrisInterface* m_pExternInterface = nullptr;

public:

	inline IIrisInterface* GetExternInterface() { return m_pExternInterface; }

	inline const IrisInternString& GetInterfaceName() { return m_strInterfaceName; }
	inline IIrisObject* GetInterfaceObject() { return m_pInterfaceObject; }

	IrisInterface(const IrisInternString& strInterfaceName, IrisModule* pUpperModule = nullptr);
	IrisInterface::~IrisInterface();

	void ClearJointingInterfaces();

	void AddInterface(IrisInterface* pInterface);
	void AddInterfaceFunctionDeclare(const IrisInternString& strFunctionName, int m_nParameterAmount, bool bHaveHaveVariableParameter = false);

	friend class IrisClass;
	friend class IrisModule;
	friend class IrisObject;
	friend class IrisInterpreter;
	friend class IrisStatement;
	friend class IrisGC;
};

#endif
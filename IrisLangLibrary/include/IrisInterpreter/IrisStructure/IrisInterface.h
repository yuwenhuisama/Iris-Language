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
#ifdef IR_USE_STL_STRING
		string m_strInterfaceName = "";
#else
		IrisInternString m_strInterfaceName = "";
#endif // IR_USE_STL_STRING
		int m_nParameterAmount = 0;
		bool m_bHaveVariableParameter = false;
#ifdef IR_USE_STL_STRING
		InterfaceFunctionDeclare(const string& strInterfaceName, int nParameterAmount, bool b_HaveVariableParameter) : m_strInterfaceName(strInterfaceName), m_nParameterAmount(nParameterAmount), m_bHaveVariableParameter(b_HaveVariableParameter) { }
#else
		InterfaceFunctionDeclare(const IrisInternString& strInterfaceName, int nParameterAmount, bool b_HaveVariableParameter) : m_strInterfaceName(strInterfaceName), m_nParameterAmount(nParameterAmount), m_bHaveVariableParameter(b_HaveVariableParameter) { }
#endif // IR_USE_STL_STRING
		InterfaceFunctionDeclare() { }
	};

private:

#ifdef IR_USE_STL_STRING
	typedef unordered_map<string, InterfaceFunctionDeclare> _InterfaceFunctionDeclareMap;
	typedef pair<string, InterfaceFunctionDeclare> _InterfaceFunctionDeclarePair;
	typedef unordered_set<IrisInterface*> _InterfaceSet;
#else
	typedef unordered_map<IrisInternString, InterfaceFunctionDeclare, IrisInternString::IrisInerStringHash> _InterfaceFunctionDeclareMap;
	typedef pair<IrisInternString, InterfaceFunctionDeclare> _InterfaceFunctionDeclarePair;
	typedef unordered_set<IrisInterface*> _InterfaceSet;
#endif // IR_USE_STL_STRING

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

#ifdef IR_USE_STL_STRING
	inline const string& GetInterfaceName() { return m_strInterfaceName; }
#else
	inline const IrisInternString& GetInterfaceName() { return m_strInterfaceName; }
#endif // IR_USE_STL_STRING
	inline IIrisObject* GetInterfaceObject() { return m_pInterfaceObject; }

#ifdef IR_USE_STL_STRING
	IrisInterface(const string& strInterfaceName, IrisModule* pUpperModule = nullptr);
#else
	IrisInterface(const IrisInternString& strInterfaceName, IrisModule* pUpperModule = nullptr);
#endif // IR_USE_STL_STRING
	IrisInterface::~IrisInterface();

	void ClearJointingInterfaces();

	void AddInterface(IrisInterface* pInterface);
#ifdef IR_USE_STL_STRING
	void AddInterfaceFunctionDeclare(const string& strFunctionName, int m_nParameterAmount, bool bHaveHaveVariableParameter = false);
#else
	void AddInterfaceFunctionDeclare(const IrisInternString& strFunctionName, int m_nParameterAmount, bool bHaveHaveVariableParameter = false);
#endif // IR_USE_STL_STRING

	friend class IrisClass;
	friend class IrisModule;
	friend class IrisObject;
	friend class IrisInterpreter;
	friend class IrisStatement;
	friend class IrisGC;
};

#endif
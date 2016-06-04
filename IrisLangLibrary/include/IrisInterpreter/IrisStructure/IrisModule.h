#ifndef _H_IRISMODULE_
#define _H_IRISMODULE_

#include <string>
#include <unordered_map>
#include <unordered_set>
using namespace std;

#include "IrisUnil/IrisValue.h"
#include "IrisObject.h"
#include "IrisMethod.h"
#include "IrisClass.h"
#include "IrisInterface.h"
#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisThread/IrisWLLock.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"
#include "IrisUnil/IrisInternString.h"

#ifdef IR_USE_MEM_POOL
class IrisModule : public IrisObjectMemoryPoolInterface<IrisModule, POOLID_IrisModule> 
#else
class IrisModule
#endif
{
private:
	typedef unordered_map<IrisInternString, IrisMethod*, IrisInternString::IrisInerStringHash> _MethodHash;
	typedef pair<IrisInternString, IrisMethod*> _MethodPair;

	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _VariableHash;
	typedef pair<IrisInternString, IrisValue> _VariablePair;

	typedef unordered_map<IrisInternString, IrisModule*, IrisInternString::IrisInerStringHash> _ModuleHash;
	typedef pair<IrisInternString, IrisModule*> _ModulePair;

	typedef unordered_set<IrisModule*> _ModuleSet;
	typedef unordered_set<IrisClass*> _ClassSet;
	typedef unordered_set<IrisInterface*> _InterfaceSet;

private:

	IIrisModule* m_pExternModule = nullptr;

	IrisInternString m_strModuleName = "";
	IrisModule* m_pUpperModule = nullptr;
	_MethodHash m_hsInstanceMethods;
	_VariableHash m_hsClassVariables;
	_VariableHash m_hsConstances;

	_ModuleHash m_hsInvolveModules;

	_ModuleSet m_hsModules;
	_InterfaceSet m_hsInvolvedInterfaces;
	_ClassSet m_hsClasses;

	void _SearchModuleMethod(const IrisInternString& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _SearchConstance(const IrisInternString& strConstanceName, IrisModule* pCurModule, IrisValue** pValue);
	void _SearchClassVariable(const IrisInternString& strVariableName, IrisModule* pCurModule, IrisValue** pValue);

	void _GetModuleInstanceMethod(const IrisInternString& strMethodName, IrisModule* pCurModule, IrisMethod** ppResultMethod);

private:
	IIrisObject* m_pModuleObject = nullptr;

	IrisWLLock m_iwlInstanceMethodWLLock;
	IrisWLLock m_iwlClassAddingWLLock;
	IrisWLLock m_iwlModuleAddingWLLock;
	IrisWLLock m_iwlInterfaceAddingWLLock;
	IrisWLLock m_iwlModuleInvolvingWLLock;

	IrisWLLock m_iwlClassVariableWLLock;
	IrisWLLock m_iwlModuleClassVariableWLLock;
	IrisWLLock m_iwlModuleConstanceWLLock;

public:

	virtual IIrisModule* GetExternModule() { return m_pExternModule; }
	virtual const IrisValue&  SearchConstance(const IrisInternString& strConstName, bool& bResult);
	virtual void AddModule(IrisModule* pModule);
	virtual void AddInvolvingModule(IrisModule* pModule);
	//virtual void AddInstanceVariable(const string& strClassVariableName);
	virtual const IrisValue& SearchClassVariable(const IrisInternString& strClassVariableName, bool& bResult);

	virtual IrisValue CallClassMethod(const IrisInternString& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);

	virtual void ResetAllMethodsObject();

	virtual IrisMethod* GetCurrentModuleMethod(const IrisInternString& strMethodName);

	virtual const IrisValue& GetCurrentModuleClassVariable(const IrisInternString& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentModuleConstance(const IrisInternString& strConstanceName, bool& bResult);

	IrisModule(const IrisInternString& strModuleName, IrisModule* pUpperModule = nullptr);
	virtual ~IrisModule();

	virtual void NativeModuleDefine() {};

	// inlines
public:
	virtual void ClearInvolvingModules();
	virtual void ClearJointedInterfaces();

	virtual void AddConstance(const IrisInternString& strConstName, const IrisValue& ivInitialValue);

	virtual void AddClassMethod(IrisMethod* pMethod);

	virtual void AddInstanceMethod(IrisMethod* pMethod);

	virtual void AddClassVariable(const IrisInternString& strClassVariableName);

	virtual void AddClassVariable(const IrisInternString& strClassVariableName, const IrisValue& ivInitialValue);

	virtual void AddClass(IrisClass* pClass);

	virtual void AddInvolvedInterface(IrisInterface* pInterface);

	virtual IrisClass* GetClass(const IrisInternString& strClassName);

	virtual IrisInterface* GetInterface(const IrisInternString& strInterfaceName);

	virtual IrisMethod* GetModuleInstanceMethod(const IrisInternString& strMethodName);

	inline IIrisObject* GetModuleObject() { return  m_pModuleObject; }

	inline virtual const IrisInternString& GetModuleName() { return m_strModuleName; }
	inline virtual IrisModule* GetUpperModule() { return m_pUpperModule; }

	inline virtual _VariableHash& GetConstances() { return m_hsConstances; }
	inline virtual _VariableHash& GetClassVariables() { return m_hsClassVariables; }
	inline virtual _ModuleSet& GetModules() { return m_hsModules; }

	//inline virtual _MethodHash& GetClassMethods() { return m_hsClassMethods; }
	inline virtual _MethodHash& GetInstanceMethods() { return m_hsInstanceMethods; }
	
	friend class IrisGC;
	friend class IrisInterpreter;
};

#endif
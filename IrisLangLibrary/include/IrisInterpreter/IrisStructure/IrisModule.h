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

#ifdef IR_USE_STL_STRING
	typedef unordered_map<string, IrisMethod*> _MethodHash;
	typedef pair<string, IrisMethod*> _MethodPair;

	typedef unordered_map<string, IrisValue> _VariableHash;
	typedef pair<string, IrisValue> _VariablePair;

	typedef unordered_map<string, IrisModule*> _ModuleHash;
	typedef pair<string, IrisModule*> _ModulePair;

	typedef unordered_set<IrisModule*> _ModuleSet;
	typedef unordered_set<IrisClass*> _ClassSet;
	typedef unordered_set<IrisInterface*> _InterfaceSet;
#else
	typedef unordered_map<IrisInternString, IrisMethod*, IrisInternString::IrisInerStringHash> _MethodHash;
	typedef pair<IrisInternString, IrisMethod*> _MethodPair;

	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _VariableHash;
	typedef pair<IrisInternString, IrisValue> _VariablePair;

	typedef unordered_map<IrisInternString, IrisModule*, IrisInternString::IrisInerStringHash> _ModuleHash;
	typedef pair<IrisInternString, IrisModule*> _ModulePair;

	typedef unordered_set<IrisModule*> _ModuleSet;
	typedef unordered_set<IrisClass*> _ClassSet;
	typedef unordered_set<IrisInterface*> _InterfaceSet;
#endif // IR_USE_STL_STRING

private:

	IIrisModule* m_pExternModule = nullptr;

#ifdef IR_USE_STL_STRING
	string m_strModuleName = "";
#else
	IrisInternString m_strModuleName = "";
#endif // IR_USE_STL_STRING
	IrisModule* m_pUpperModule = nullptr;
	_MethodHash m_hsInstanceMethods;
	_VariableHash m_hsClassVariables;
	_VariableHash m_hsConstances;

	_ModuleHash m_hsInvolveModules;

	_ModuleSet m_hsModules;
	_InterfaceSet m_hsInvolvedInterfaces;
	_ClassSet m_hsClasses;

#ifdef IR_USE_STL_STRING
	void _SearchModuleMethod(const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _SearchConstance(const string& strConstanceName, IrisModule* pCurModule, IrisValue** pValue);
	void _GetModuleInstanceMethod(const string& strMethodName, IrisModule* pCurModule, IrisMethod** ppResultMethod);
	void _SearchClassVariable(const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue);
#else
	void _SearchModuleMethod(const IrisInternString& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _SearchConstance(const IrisInternString& strConstanceName, IrisModule* pCurModule, IrisValue** pValue);
	void _SearchClassVariable(const IrisInternString& strVariableName, IrisModule* pCurModule, IrisValue** pValue);
	void _GetModuleInstanceMethod(const IrisInternString& strMethodName, IrisModule* pCurModule, IrisMethod** ppResultMethod);
#endif // IR_USE_STL_STRING

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
#ifdef IR_USE_STL_STRING
	virtual const IrisValue&  SearchConstance(const string& strConstName, bool& bResult);
#else
	virtual const IrisValue&  SearchConstance(const IrisInternString& strConstName, bool& bResult);
#endif // IR_USE_STL_STRING
	virtual void AddModule(IrisModule* pModule);
	virtual void AddInvolvingModule(IrisModule* pModule);
	//virtual void AddInstanceVariable(const string& strClassVariableName);
#ifdef IR_USE_STL_STRING
	virtual const IrisValue& SearchClassVariable(const string& strClassVariableName, bool& bResult);
#else
	virtual const IrisValue& SearchClassVariable(const IrisInternString& strClassVariableName, bool& bResult);
#endif // IR_USE_STL_STRING

#ifdef IR_USE_STL_STRING
	virtual IrisValue CallClassMethod(const string& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);
#else
	virtual IrisValue CallClassMethod(const IrisInternString& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);
#endif // IR_USE_STL_STRING

	virtual void ResetAllMethodsObject();

#ifdef IR_USE_STL_STRING
	virtual IrisMethod* GetCurrentModuleMethod(const string& strMethodName);
	virtual const IrisValue& GetCurrentModuleClassVariable(const string& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentModuleConstance(const string& strConstanceName, bool& bResult);
	IrisModule(const string& strModuleName, IrisModule* pUpperModule = nullptr);

#else
	virtual IrisMethod* GetCurrentModuleMethod(const IrisInternString& strMethodName);
	virtual const IrisValue& GetCurrentModuleClassVariable(const IrisInternString& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentModuleConstance(const IrisInternString& strConstanceName, bool& bResult);
	IrisModule(const IrisInternString& strModuleName, IrisModule* pUpperModule = nullptr);
#endif // IR_USE_STL_STRING

	virtual ~IrisModule();

	virtual void NativeModuleDefine() {};

	// inlines
public:
	virtual void ClearInvolvingModules();
	virtual void ClearJointedInterfaces();

#ifdef IR_USE_STL_STRING
	virtual void AddConstance(const string& strConstName, const IrisValue& ivInitialValue);
#else
	virtual void AddConstance(const IrisInternString& strConstName, const IrisValue& ivInitialValue);
#endif // IR_USE_STL_STRING

	virtual void AddClassMethod(IrisMethod* pMethod);

	virtual void AddInstanceMethod(IrisMethod* pMethod);

#ifdef IR_USE_STL_STRING
	virtual void AddClassVariable(const string& strClassVariableName);
	virtual void AddClassVariable(const string& strClassVariableName, const IrisValue& ivInitialValue);
#else
	virtual void AddClassVariable(const IrisInternString& strClassVariableName);
	virtual void AddClassVariable(const IrisInternString& strClassVariableName, const IrisValue& ivInitialValue);
#endif // IR_USE_STL_STRING


	virtual void AddClass(IrisClass* pClass);

	virtual void AddInvolvedInterface(IrisInterface* pInterface);


#ifdef IR_USE_STL_STRING
	virtual IrisClass* GetClass(const string& strClassName);
	virtual IrisInterface* GetInterface(const string& strInterfaceName);
	virtual IrisMethod* GetModuleInstanceMethod(const string& strMethodName);
#else
	virtual IrisClass* GetClass(const IrisInternString& strClassName);
	virtual IrisInterface* GetInterface(const IrisInternString& strInterfaceName);
	virtual IrisMethod* GetModuleInstanceMethod(const IrisInternString& strMethodName);
#endif // IR_USE_STL_STRING

	inline IIrisObject* GetModuleObject() { return  m_pModuleObject; }

#ifdef IR_USE_STL_STRING
	inline virtual const string& GetModuleName() { return m_strModuleName; }
#else
	inline virtual const IrisInternString& GetModuleName() { return m_strModuleName; }
#endif // IR_USE_STL_STRING
	inline virtual IrisModule* GetUpperModule() { return m_pUpperModule; }

	inline virtual _VariableHash& GetConstances() { return m_hsConstances; }
	inline virtual _VariableHash& GetClassVariables() { return m_hsClassVariables; }
#ifdef IR_USE_STL_STRING
	inline virtual _ModuleHash& GetModules() { return m_hsModules; }
#else
	inline virtual _ModuleSet& GetModules() { return m_hsModules; }
#endif // IR_USE_STL_STRING

	//inline virtual _MethodHash& GetClassMethods() { return m_hsClassMethods; }
	inline virtual _MethodHash& GetInstanceMethods() { return m_hsInstanceMethods; }
	
	friend class IrisGC;
	friend class IrisInterpreter;
};

#endif
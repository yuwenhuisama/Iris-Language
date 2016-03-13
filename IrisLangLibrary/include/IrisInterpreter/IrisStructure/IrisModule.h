#ifndef _H_IRISMODULE_
#define _H_IRISMODULE_

#include <string>
#include <unordered_map>
using namespace std;

#include "IrisUnil/IrisValue.h"
#include "IrisObject.h"
#include "IrisMethod.h"
#include "IrisClass.h"
#include "IrisInterface.h"
#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisThread/IrisWLLock.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"

#ifdef IR_USE_MEM_POOL
class IrisModule : public IrisObjectMemoryPoolInterface<IrisModule, POOLID_IrisModule> 
#else
class IrisModule
#endif
{
private:
	typedef unordered_map<string, IrisMethod*> _MethodHash;
	typedef pair<string, IrisMethod*> _MethodPair;

	typedef unordered_map<string, IrisValue> _VariableHash;
	typedef pair<string, IrisValue> _VariablePair;

	typedef unordered_map<string, IrisModule*> _ModuleHash;
	typedef pair<string, IrisModule*> _ModulePair;

	typedef unordered_map<string, IrisClass*> _ClassHash;
	typedef pair<string, IrisClass*> _ClassPair;

	typedef unordered_map<string, IrisInterface*> _InterfaceHash;
	typedef pair<string, IrisInterface*> _InterfacePair;

public:
	//enum class SearchMethodType {
	//	InstanceMethod = 0,
	//	ClassMethod,
	//};

private:

	IIrisModule* m_pExternModule = nullptr;

	string m_strModuleName = "";
	IrisModule* m_pUpperModule = nullptr;
	_ModuleHash m_hsModules;
	//_MethodHash m_hsClassMethods;
	_MethodHash m_hsInstanceMethods;
	//_VariableHash m_hsInstanceVariables;
	_VariableHash m_hsClassVariables;
	_VariableHash m_hsConstances;

	_InterfaceHash m_hsInvolvedInterfaces;

	_ClassHash m_hsClasses;

	void _SearchModuleMethod(const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _SearchConstance(const string& strConstanceName, IrisModule* pCurModule, IrisValue** pValue);
	void _SearchClassVariable(const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue);

	void _GetModuleInstanceMethod(const string& strMethodName, IrisModule* pCurModule, IrisMethod** ppResultMethod);

private:
	IIrisObject* m_pModuleObject = nullptr;

	IrisWLLock m_iwlInstanceMethodWLLock;
	IrisWLLock m_iwlClassAddingWLLock;
	IrisWLLock m_iwlModuleAddingWLLock;
	IrisWLLock m_iwlInterfaceAddingWLLock;

	IrisWLLock m_iwlClassVariableWLLock;
	IrisWLLock m_iwlModuleClassVariableWLLock;
	IrisWLLock m_iwlModuleConstanceWLLock;

public:

	virtual IIrisModule* GetExternModule() { return m_pExternModule; }
	virtual const IrisValue&  SearchConstance(const string& strConstName, bool& bResult);
	virtual void AddModule(IrisModule* pModule);
	//virtual void AddInstanceVariable(const string& strClassVariableName);
	virtual const IrisValue& SearchClassVariable(const string& strClassVariableName, bool& bResult);

	virtual IrisValue CallClassMethod(const string& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);

	virtual void ResetAllMethodsObject();

	virtual IrisMethod* GetCurrentModuleMethod(const string& strMethodName);

	virtual const IrisValue& GetCurrentModuleClassVariable(const string& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentModuleConstance(const string& strConstanceName, bool& bResult);

	IrisModule(const string& strModuleName, IrisModule* pUpperModule = nullptr);
	virtual ~IrisModule();

	virtual void NativeModuleDefine() {};

	// inlines
public:
	virtual void ClearInvolvingModules();
	virtual void ClearJointedInterfaces();

	virtual void AddConstance(const string& strConstName, const IrisValue& ivInitialValue);

	virtual void AddClassMethod(IrisMethod* pMethod);

	virtual void AddInstanceMethod(IrisMethod* pMethod);

	virtual void AddClassVariable(const string& strClassVariableName);

	virtual void AddClassVariable(const string& strClassVariableName, const IrisValue& ivInitialValue);

	virtual void AddClass(IrisClass* pClass);

	virtual void AddInvolvedInterface(IrisInterface* pInterface);

	virtual IrisClass* GetClass(const string& strClassName);

	virtual IrisInterface* GetInterface(const string& strInterfaceName);

	virtual IrisMethod* GetModuleInstanceMethod(const string& strMethodName);

	inline IIrisObject* GetModuleObject() { return  m_pModuleObject; }

	inline virtual const string& GetModuleName() { return m_strModuleName; }
	inline virtual IrisModule* GetUpperModule() { return m_pUpperModule; }

	inline virtual _VariableHash& GetConstances() { return m_hsConstances; }
	inline virtual _VariableHash& GetClassVariables() { return m_hsClassVariables; }
	inline virtual _ModuleHash& GetModules() { return m_hsModules; }

	//inline virtual _MethodHash& GetClassMethods() { return m_hsClassMethods; }
	inline virtual _MethodHash& GetInstanceMethods() { return m_hsInstanceMethods; }
	
	friend class IrisGC;
	friend class IrisInterpreter;
};

#endif
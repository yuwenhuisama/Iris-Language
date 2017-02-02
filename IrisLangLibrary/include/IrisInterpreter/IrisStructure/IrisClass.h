#ifndef _H_IRISCLASS_
#define _H_IRISCLASS_

#include "../../IrisCompileConfigure.h"

#include "IrisUnil/IrisValue.h"
#include "IrisInterface.h"
#include "IrisMethod.h"
#include "IrisInterpreter.h"
#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisThread/IrisWLLock.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"

#include "IrisInterfaces/IIrisClass.h"
#include "IrisUnil/IrisInternString.h"

#include <string>
#include <map>
#include <unordered_map>
#include <unordered_set>
using namespace std;

//class IrisInterface;
class IrisMethod;
class IrisModule;
class IrisInterpreter;

class IIrisClass;
class IIrisValues;

#if IR_USE_MEM_POOL
class IrisClass : public IrisObjectMemoryPoolInterface<IrisClass, POOLID_IrisClass>
#else
class IrisClass
#endif
{
private:

#if IR_USE_STL_STRING
	typedef unordered_map<string, IrisMethod*> _MethodHash;
	typedef pair<string, IrisMethod*> _MethodPair;
	typedef unordered_map<string, IrisValue> _VariableHash;
	typedef pair<string, IrisValue> _VariablePair;

	typedef unordered_set<IrisInterface*> _InterfaceSet;
	typedef unordered_set<IrisModule*> _ModuleSet;

	typedef unordered_map<string, IrisInterface::InterfaceFunctionDeclare> _InterfaceFunctionDeclareMap;
	typedef pair<string, IrisInterface::InterfaceFunctionDeclare> _InterfaceFunctionDeclarePair;
#else
	typedef unordered_map<IrisInternString, IrisMethod*, IrisInternString::IrisInerStringHash> _MethodHash;
	typedef pair<IrisInternString, IrisMethod*> _MethodPair;
	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _VariableHash;
	typedef pair<IrisInternString, IrisValue> _VariablePair;

	typedef unordered_set<IrisInterface*> _InterfaceSet;
	typedef unordered_set<IrisModule*> _ModuleSet;

	typedef unordered_map<IrisInternString, IrisInterface::InterfaceFunctionDeclare, IrisInternString::IrisInerStringHash> _InterfaceFunctionDeclareMap;
	typedef pair<IrisInternString, IrisInterface::InterfaceFunctionDeclare> _InterfaceFunctionDeclarePair;
#endif // IR_USE_STL_STRING

	typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:
	enum class SearchVariableType {
		Constance = 0,
		ClassInstance,
	};

	enum class ClassType {
		NormalClass = 0,
		ClassClass,
		ModuleClass,
		InterfaceClass,
		ObjectClass,
	};

protected:
	IIrisClass* m_pExternClass = nullptr;
#if IR_USE_STL_STRING
	string m_strClassName = "";
#else
	IrisInternString m_strClassName = "";
#endif // IR_USE_STL_STRING
	
	IrisClass* m_pSuperClass = nullptr;
	IrisModule* m_pUpperModule = nullptr;
	_InterfaceSet m_hsInterfaces;
	_ModuleSet m_hsModules;
	_MethodHash m_hsInstanceMethods;
	_VariableHash m_hsClassVariables;
	_VariableHash m_hsConstances;
	bool m_bIsCompleteClass = false;;

	IIrisObject* m_pClassObject = nullptr;

	ClassType m_eClassType = ClassType::NormalClass;

	IrisWLLock m_iwlInstanceMethodWLLock;

	IrisWLLock m_iwlClassClassVariableWLLock;
	IrisWLLock m_iwlClassConstanceWLLock;

	IrisWLLock m_iwlModuleAddingWLLock;
	IrisWLLock m_iwlInterfaceAddingWLLock;

private:
	void _FunctionCollect(IrisInterface* pInterface, _InterfaceFunctionDeclareMap& mpFunctionDeclare);
	bool _FunctionAchieved();

#if IR_USE_STL_STRING
	void _ModuleMethodSearch(const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _ClassModuleMethodSearch(IrisClass* pCurClass, const string& strMethodName, IrisMethod** ppMethod);
	void _SearchModuleConstance(SearchVariableType eType, const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue);
#else
	void _ModuleMethodSearch(const IrisInternString& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _ClassModuleMethodSearch(IrisClass* pCurClass, const IrisInternString& strMethodName, IrisMethod** ppMethod);
	void _SearchModuleConstance(SearchVariableType eType, const IrisInternString& strVariableName, IrisModule* pCurModule, IrisValue** pValue);
#endif // IR_USE_STL_STRING

public:

	inline virtual IIrisClass* GetExternClass() { return m_pExternClass; }
	
	inline virtual bool IsClassClass() { return m_eClassType == ClassType::ClassClass; }
	inline virtual bool IsObjectClass() { return m_eClassType == ClassType::ObjectClass; }
	inline virtual bool IsModuleClass() { return m_eClassType == ClassType::ModuleClass; }
	inline virtual bool IsInterfaceClass() { return m_eClassType == ClassType::InterfaceClass; }
	inline virtual bool IsNormalClass() {
		return m_eClassType != ClassType::ClassClass
			&& m_eClassType != ClassType::ModuleClass
			&& m_eClassType != ClassType::InterfaceClass;
										}

	inline virtual void SetCompleted(bool bFlag) { m_bIsCompleteClass = bFlag; }

	virtual void ClearInvolvingModules();

	virtual void ClearJointingInterfaces();
#if IR_USE_STL_STRING
	virtual void AddConstance(const string& strConstName, const IrisValue& ivInitialValue);
	virtual const IrisValue&  SearchConstance(const string& strConstName, bool& bResult);
#else
	virtual void AddConstance(const IrisInternString& strConstName, const IrisValue& ivInitialValue);
	virtual const IrisValue&  SearchConstance(const IrisInternString& strConstName, bool& bResult);
#endif // IR_USE_STL_STRING

	virtual void AddClassMethod(IrisMethod* pMethod);

	virtual void AddInstanceMethod(IrisMethod* pMethod);

#if IR_USE_STL_STRING
	virtual void AddClassVariable(const string& strClassVariableName);
	virtual void AddClassVariable(const string& strClassVariableName, const IrisValue& ivInitialValue);

	virtual void AddSetter(const string& strProperName, IrisNativeFunction pfMethod);
	virtual void AddGetter(const string& strProperName, IrisNativeFunction pfMethod);
#else
	virtual void AddClassVariable(const IrisInternString& strClassVariableName);
	virtual void AddClassVariable(const IrisInternString& strClassVariableName, const IrisValue& ivInitialValue);

	virtual void AddSetter(const IrisInternString& strProperName, IrisNativeFunction pfMethod);
	virtual void AddGetter(const IrisInternString& strProperName, IrisNativeFunction pfMethod);
#endif // IR_USE_STL_STRING

	virtual void AddInterface(IrisInterface* pInterface);
	virtual void AddModule(IrisModule* pModule);
	
#if IR_USE_STL_STRING
	virtual const IrisValue& SearchClassVariable(const string& strClassVariableName, bool& bResult);
	virtual const string& GetClassName() { return m_strClassName; }
	virtual IrisValue CallClassMethod(const string& strMethodName, IrisContextEnvironment* pContexEnvironment, IrisValues* ivParameters, CallerSide eSide);
	virtual IrisMethod* GetMethod(const string& strMethodName, bool& bIsCurClassMethod);
	virtual IrisMethod* GetCurrentClassMethod(const string& strMethodName);
#else
	virtual const IrisValue& SearchClassVariable(const IrisInternString& strClassVariableName, bool& bResult);
	virtual const IrisInternString& GetClassName() { return m_strClassName; }
	virtual IrisValue CallClassMethod(const IrisInternString& strMethodName, IrisContextEnvironment* pContexEnvironment, IrisValues* ivParameters, CallerSide eSide);
	virtual IrisMethod* GetMethod(const IrisInternString& strMethodName, bool& bIsCurClassMethod);
	virtual IrisMethod* GetCurrentClassMethod(const IrisInternString& strMethodName);
#endif // IR_USE_STL_STRING
	
	virtual IrisValue CreateInstance(IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment);

#if IR_USE_STL_STRING
	virtual const IrisValue& GetCurrentClassClassVariable(const string& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentClassConstance(const string& strConstanceName, bool& bResult);
#else
	virtual const IrisValue& GetCurrentClassClassVariable(const IrisInternString& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentClassConstance(const IrisInternString& strConstanceName, bool& bResult);
#endif // IR_USE_STL_STRING

	virtual void ResetNativeObject();
	virtual void SetSuperClass(IrisClass* pSuperClass) { m_pSuperClass = pSuperClass; }
	virtual IrisClass* GetSuperClass() { return m_pSuperClass; }

#if IR_USE_STL_STRING
	IrisClass(const string& strClassName, IrisClass* pSuperClass = nullptr, IrisModule* pUpperModule = nullptr, IIrisClass* pExternClass = nullptr);
#else
	IrisClass(const IrisInternString& strClassName, IrisClass* pSuperClass = nullptr, IrisModule* pUpperModule = nullptr, IIrisClass* pExternClass = nullptr);
#endif // IR_USE_STL_STRING
	virtual ~IrisClass();

	virtual void ResetAllMethodsObject();

	//Getter Setter
	inline IrisModule* GetUpperModule() {
		return m_pUpperModule;
	}

	inline void SetUpperModule(IrisModule* pModule) {
		m_pUpperModule = pModule;
	}

	inline IIrisObject* GetClassObject() { return m_pClassObject; }

	inline _ModuleSet& GetModules() { return m_hsModules; }

	friend class IrisGC;
	friend class IrisInterpreter;
};

#endif
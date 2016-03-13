#ifndef _H_IRISCLASS_
#define _H_IRISCLASS_

#include "IrisUnil/IrisValue.h"
#include "IrisInterface.h"
#include "IrisMethod.h"
#include "IrisInterpreter.h"
#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisThread/IrisWLLock.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"

#include "IrisInterfaces/IIrisClass.h"

#include <string>
#include <map>
#include <unordered_map>
using namespace std;

//class IrisInterface;
class IrisMethod;
class IrisModule;
class IrisInterpreter;

class IIrisClass;
class IIrisValues;

#ifdef IR_USE_MEM_POOL
class IrisClass : public IrisObjectMemoryPoolInterface<IrisClass, POOLID_IrisClass>
#else
class IrisClass
#endif
{
private:
	typedef unordered_map<std::string, IrisMethod*> _MethodHash;
	typedef pair<string, IrisMethod*> _MethodPair;
	typedef unordered_map<string, IrisValue> _VariableHash;
	typedef pair<string, IrisValue> _VariablePair;
	typedef unordered_map<string, IrisInterface*> _InterfaceHash;
	typedef pair<string, IrisInterface*> _InterfacePair;
	typedef unordered_map<string, IrisModule*> _ModuleHash;
	typedef pair<string, IrisModule*> _ModulePair;
	typedef unordered_map<string, IrisInterface::InterfaceFunctionDeclare> _InterfaceFunctionDeclareMap;
	typedef pair<string, IrisInterface::InterfaceFunctionDeclare> _InterfaceFunctionDeclarePair;

	typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:
	enum class SearchMethodType {
		InstanceMethod = 0,
		ClassMethod,
	};

	enum class SearchVariableType {
		Constance = 0,
		ClassInstance,
	};

	enum class ClassType {
		NormalClass = 0,
		ClassClass,
		ModuleClass,
		InterfaceClass,
	};

protected:
	IIrisClass* m_pExternClass = nullptr;
	string m_strClassName = "";
	IrisClass* m_pSuperClass = nullptr;
	IrisModule* m_pUpperModule = nullptr;
	_InterfaceHash m_hsInterfaces;
	_ModuleHash m_hsModules;
	_MethodHash m_hsClassMethods;
	_MethodHash m_hsInstanceMethods;
	_VariableHash m_hsClassVariables;
	_VariableHash m_hsConstances;
	bool m_bIsCompleteClass = false;;

	IIrisObject* m_pClassObject = nullptr;

	ClassType m_eClassType = ClassType::NormalClass;

	IrisWLLock m_iwlInstanceMethodWLLock;
	IrisWLLock m_iwlClassMethodWLLock;

	IrisWLLock m_iwlClassClassVariableWLLock;
	IrisWLLock m_iwlClassConstanceWLLock;

	IrisWLLock m_iwlModuleAddingWLLock;
	IrisWLLock m_iwlInterfaceAddingWLLock;

private:
	void _FunctionCollect(IrisInterface* pInterface, _InterfaceFunctionDeclareMap& mpFunctionDeclare);
	bool _FunctionAchieved();

	void _ModuleMethodSearch(SearchMethodType eSearchType, const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod);
	void _ClassModuleMethodSearch(SearchMethodType eSearchType, IrisClass* pCurClass, const string& strMethodName, IrisMethod** ppMethod);
	void _SearchModuleConstance(SearchVariableType eType, const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue);

public:

	inline virtual IIrisClass* GetExternClass() { return m_pExternClass; }

	inline virtual bool IsClassClass() { return m_eClassType == ClassType::ClassClass; }
	inline virtual bool IsNormalClass() { return m_eClassType == ClassType::NormalClass; }
	inline virtual bool IsModuleClass() { return m_eClassType == ClassType::ModuleClass; }
	inline virtual bool IsInterfaceClass() { return m_eClassType == ClassType::InterfaceClass; }

	inline virtual void SetCompleted(bool bFlag) { m_bIsCompleteClass = bFlag; }

	virtual void ClearInvolvingModules();

	virtual void ClearJointingInterfaces();

	virtual void AddConstance(const string& strConstName, const IrisValue& ivInitialValue);
	virtual const IrisValue&  SearchConstance(const string& strConstName, bool& bResult);

	virtual void AddClassMethod(IrisMethod* pMethod);

	virtual void AddInstanceMethod(IrisMethod* pMethod);

	virtual void AddClassVariable(const string& strClassVariableName);
	virtual void AddClassVariable(const string& strClassVariableName, const IrisValue& ivInitialValue);

	virtual void AddSetter(const string& strProperName, IrisNativeFunction pfMethod);
	virtual void AddGetter(const string& strProperName, IrisNativeFunction pfMethod);

	virtual void AddInterface(IrisInterface* pInterface);
	virtual void AddModule(IrisModule* pModule);
	
	virtual const IrisValue& SearchClassVariable(const string& strClassVariableName, bool& bResult);
	
	virtual IrisValue CreateInstance(IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment);
	//virtual IrisValue CreateInstanceFromLiteral(char* pLiteral);
	//virtual IrisValue CreateInstanceByInstantValue(int nValue) { IrisValue ivValue; ivValue.SetIrisObject(nullptr); return ivValue; }
	//virtual IrisValue CreateInstanceByInstantValue(double dValue) { IrisValue ivValue; ivValue.SetIrisObject(nullptr); return ivValue; }
	//virtual IrisValue CreateInstanceByInstantValue(const string& strString) { IrisValue ivValue; ivValue.SetIrisObject(nullptr); return ivValue; }

	virtual const string& GetClassName() { return m_strClassName; }
	virtual IrisValue CallClassMethod(const string& strMethodName, IrisContextEnvironment* pContexEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);

	virtual IrisMethod* GetMethod(SearchMethodType eSearchType, const string& strMethodName, bool& bIsCurClassMethod);

	virtual IrisMethod* GetCurrentClassMethod(SearchMethodType eSearchType, const string& strMethodName);

	virtual const IrisValue& GetCurrentClassClassVariable(const string& strVariableName, bool& bResult);
	virtual const IrisValue& GetCurrentClassConstance(const string& strConstanceName, bool& bResult);

	virtual void ResetNativeObject();
	virtual void SetSuperClass(IrisClass* pSuperClass) { m_pSuperClass = pSuperClass; }
	virtual IrisClass* GetSuperClass() { return m_pSuperClass; }

	IrisClass(const string& strClassName, IrisClass* pSuperClass = nullptr, IrisModule* pUpperModule = nullptr);
	virtual ~IrisClass();

	// 为字面量准备的（Integer/Float/String）
	//virtual void* GetLiteralObject(char* pLiterral) { return nullptr; }

	virtual void ResetAllMethodsObject();

	//virtual int GetTrustteeSize(void* pNativePointer) { return 0; }

	//virtual void* NativeAlloc() { return nullptr; }
	//virtual void NativeFree(void* pNativePointer) {}
	//virtual void NativeClassDefine() {}
	//virtual void Mark(void* pNativePointer) {}

	//Getter Setter
	inline IrisModule* GetUpperModule() {
		return m_pUpperModule;
	}

	inline void SetUpperModule(IrisModule* pModule) {
		m_pUpperModule = pModule;
	}

	inline IIrisObject* GetClassObject() { return m_pClassObject; }

	friend class IrisGC;
	friend class IrisInterpreter;
};

#endif
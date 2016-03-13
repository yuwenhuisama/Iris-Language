#ifndef _H_IIRISCLASS_
#define _H_IIRISCLASS_

#include "IrisUnil/IrisValue.h"

class IIrisMethod;
class IIrisModule;
class IIrisInterface;
class IIrisContextEnvironment;
class IIrisValues;

class IrisClass;

class IIrisClass
{
private:
	IrisClass* m_pInternClass = nullptr;

public:

	typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	
	//enum class SearchMethodType {
	//	InstanceMethod = 0,
	//	ClassMethod,
	//};

	//enum class SearchVariableType {
	//	Constance = 0,
	//	ClassInstance,
	//};

public:

	IIrisClass() {}

	virtual IrisClass* GetInternClass() { return m_pInternClass; };

	virtual const char* NativeClassNameDefine() const = 0;
	virtual IIrisClass* NativeSuperClassDefine() const = 0;
	virtual IIrisModule* NativeUpperModuleDefine() const { return nullptr; };

	virtual int GetTrustteeSize(void* pNativePointer) = 0;

	virtual void* NativeAlloc() = 0;
	virtual void NativeFree(void* pNativePointer) = 0;
	virtual void NativeClassDefine() = 0;
	virtual void Mark(void* pNativePointer) { return; }

	//virtual bool IsClassClass() = 0;
	//virtual bool IsNormalClass() = 0;
	//virtual bool IsModuleClass() = 0;
	//virtual bool IsInterfaceClass() = 0;

	//virtual void SetCompleted(bool bFlag) = 0;

	//virtual void ClearInvolvingModules() = 0;
	//virtual void ClearJointingInterfaces() = 0;

	//virtual void AddConstance(const string& strConstName, const IrisValue& ivInitialValue) = 0;
	//virtual const IrisValue&  SearchConstance(const string& strConstName, bool& bResult) = 0;

	//virtual void AddClassMethod(IIrisMethod* pMethod) = 0;
	//virtual void AddInstanceMethod(IIrisMethod* pMethod) = 0;

	//virtual void AddClassVariable(const string& strClassVariableName) = 0;
	//virtual void AddClassVariable(const string& strClassVariableName, const IrisValue& ivInitialValue) = 0;

	//virtual void AddSetter(const string& strProperName, IrisNativeFunction pfMethod) = 0;
	//virtual void AddGetter(const string& strProperName, IrisNativeFunction pfMethod) = 0;

	//virtual void AddInterface(IIrisInterface* pInterface) = 0;
	//virtual void AddModule(IIrisModule* pModule) = 0;

	//virtual const IrisValue& SearchClassVariable(const string& strClassVariableName, bool& bResult) = 0;

	//virtual IrisValue CreateInstance(IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment) = 0;
	//virtual IrisValue CreateInstanceFromLiteral(char* pLiteral) = 0;
	//virtual IrisValue CreateInstanceByInstantValue(int nValue) = 0;
	//virtual IrisValue CreateInstanceByInstantValue(double dValue) = 0;
	//virtual IrisValue CreateInstanceByInstantValue(const string& strString) = 0;

	//virtual const string& GetClassName() = 0;
	//virtual IrisValue CallClassMethod(const string& strMethodName, IIrisContextEnvironment* pContexEnvironment, IIrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1) = 0;

	//virtual IIrisMethod* GetMethod(SearchMethodType eSearchType, const string& strMethodName, bool& bIsCurClassMethod) = 0;

	//virtual IIrisMethod* GetCurrentClassMethod(SearchMethodType eSearchType, const string& strMethodName) = 0;

	//virtual const IrisValue& GetCurrentClassClassVariable(const string& strVariableName, bool& bResult) = 0;
	//virtual const IrisValue& GetCurrentClassConstance(const string& strConstanceName, bool& bResult) = 0;

	//virtual void ResetNativeObject() = 0;
	//virtual void SetSuperClass(IIrisClass* pSuperClass) = 0;
	//virtual IIrisClass* GetSuperClass() = 0;

	//IIrisClass(const string& strClassName, IIrisClass* pSuperClass = nullptr, IIrisModule* pUpperModule = nullptr);

	// 为字面量准备的（Integer/Float/String）
	//virtual void* GetLiteralObject(char* pLiterral) = 0;

	//virtual void ResetAllMethodsObject() = 0;

	//Getter Setter
	//virtual IIrisModule* GetUpperModule() = 0;

	//virtual void SetUpperModule(IIrisModule* pModule) = 0;

	//virtual IIrisObject* GetClassObject() = 0;

	virtual ~IIrisClass() = 0 {};

	friend class IrisClass;
	friend class IrisInterpreter;
};

#endif
#ifndef _H_IRIDEVELOPUTIL_
#define _H_IRIDEVELOPUTIL_
#include "IrisUnil/IrisValue.h"
//#include "IrisThread/IrisThreadManager.h"
//#include <string>
//using namespace std;

class IIrisValues;
class IIrisClass;
class IIrisModule;
class IIrisInterface;
class IIrisContextEnvironment;

typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

template<class T>
T IrisDev_GetNativePointer(IrisValue& ivValue) {
	return static_cast<T>(ivValue.GetInstanceNativePointer());
}

template<class T>
T IrisDev_GetNativePointer(IIrisObject* pObject) {
	return static_cast<T>(pObject->GetNativeObject());
}

bool IrisDev_CheckClass(IrisValue& ivValue, const char* szClassName);

void IrisDev_GroanIrregularWithString(const char* szIrregularString);

int IrisDev_GetInt(IrisValue& ivValue);
double IrisDev_GetFloat(IrisValue& ivValue);
const char* IrisDev_GetString(IrisValue& ivValue);
IrisValue IrisDev_CallMethod(IrisValue& ivObj, IIrisValues* pParameters, const char* szMethodName);
IrisValue IrisDev_CreateInt(int nInteger);
IrisValue IrisDev_CreateFloat(double dDouble);
IrisValue IrisDev_CreateString(const char* nInteger);
IIrisClass* IrisDev_GetClass(const char* strClassPathName);
IIrisModule* IrisDev_GetModule(const char* strClassPathName);
IIrisInterface* IrisDev_GetInterface(const char* strClassPathName);

//IrisThreadUniqueInfo* IrisDev_GetCurrentThreadInfo();
//
//bool IrisDev_CurrentThreadIsMainThread();

const IrisValue& IrisDev_Nil();
const IrisValue& IrisDev_False();
const IrisValue& IrisDev_True();

void IrisDev_AddInstanceMethod(IIrisClass* pClass, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);
void IrisDev_AddClassMethod(IIrisClass* pClass, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);

void IrisDev_AddInstanceMethod(IIrisModule* pModule, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);
void IrisDev_AddClassMethod(IIrisModule* pModule, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);

void IrisDev_AddGetter(IIrisClass* pClass, const char* szInstanceVariableName, IrisNativeFunction pfFunction);
void IrisDev_AddSetter(IIrisClass* pClass, const char* szInstanceVariableName, IrisNativeFunction pfFunction);

void IrisDev_AddConstance(IIrisClass* pClass, const char* szConstanceName, const IrisValue& ivValue);
void IrisDev_AddConstance(IIrisModule* pModule, const char* szConstanceName, const IrisValue& ivValue);

void IrisDev_AddClassVariable(IIrisClass* pClass, const char* szVariableName, const IrisValue& ivValue);
void IrisDev_AddClassVariable(IIrisModule* pClass, const char* szVariableName, const IrisValue& ivValue);

void IrisDev_AddModule(IIrisClass* pClass, IIrisModule* pTargetModule);
void IrisDev_AddModule(IIrisModule* pModule, IIrisModule* pTargetModule);

IrisValue IrisDev_CreateInstance(IIrisClass* pClass, IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment);
IrisValue IrisDev_CreateInstanceByInstantValue(const char* szString);
IrisValue IrisDev_CreateInstanceByInstantValue(double dFloat);
IrisValue IrisDev_CreateInstanceByInstantValue(int nInteger);
IrisValue IrisDev_CreateUniqueStringInstanceByUniqueIndex(size_t nIndex);

bool IrisDev_IrregularHappened();
bool IrisDev_FatalErrorHappened();

#endif
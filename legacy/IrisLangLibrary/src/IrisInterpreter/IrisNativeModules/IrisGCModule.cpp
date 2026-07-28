#include "IrisInterpreter/IrisNativeModules/IrisGCModule.h"



IrisValue IrisGCModule::ForceStart(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisGC::CurrentGC()->ForceStart();
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisValue IrisGCModule::SetFlag(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	const IrisValue& ivFlag = static_cast<IrisValues*>(ivsValues)->GetValue(0);
	IrisGC::CurrentGC()->SetGCFlag(ivFlag == IrisInterpreter::CurrentInterpreter()->True() ? true : false);
	return IrisInterpreter::CurrentInterpreter()->Nil();;
}

IrisGCModule::IrisGCModule()
{
}


IrisGCModule::~IrisGCModule()
{
}

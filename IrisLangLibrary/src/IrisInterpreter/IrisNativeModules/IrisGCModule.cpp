#include "IrisInterpreter/IrisNativeModules/IrisGCModule.h"



IrisValue IrisGCModule::ForceStart(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisGC::CurrentGC()->ForceStart();
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisValue IrisGCModule::SetFlag(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	const IrisValue& ivFlag = ivsValues->GetValue(0);
	IrisGC::CurrentGC()->SetGCFlag(ivFlag == IrisInterpreter::CurrentInterpreter()->True() ? true : false);
	return IrisInterpreter::CurrentInterpreter()->Nil();;
}

IrisGCModule::IrisGCModule()
{
}


IrisGCModule::~IrisGCModule()
{
}

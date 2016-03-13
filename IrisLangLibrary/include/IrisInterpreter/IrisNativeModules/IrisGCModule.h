#ifndef _IRISGCCLASS_
#define _IRISGCCLASS_

#include "IrisDevHeader.h"
#include "IrisGC.h"

class IrisGCModule : public IIrisModule
{
public:

	static IrisValue ForceStart(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisGC::CurrentGC()->ForceStart();
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}

	static IrisValue SetFlag(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivFlag = ivsValues->GetValue(0);
		IrisGC::CurrentGC()->SetGCFlag((IrisValue&)ivFlag == IrisInterpreter::CurrentInterpreter()->True() ? true : false);
		return IrisInterpreter::CurrentInterpreter()->Nil();;
	}

public:
	
	const char* NativeModuleNameDefine() const {
		return "Kernel";
	}

	IIrisModule* NativeUpperModuleDefine() const {
		return nullptr;
	}

	void NativeModuleDefine() {
		IrisDevUtil::AddClassMethod(this, "start", ForceStart, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "set_flag", SetFlag, 1, false);
	};

	IrisGCModule();
	~IrisGCModule();
};

#endif
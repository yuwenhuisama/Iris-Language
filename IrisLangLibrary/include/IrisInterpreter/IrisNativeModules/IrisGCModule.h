#ifndef _IRISGCCLASS_
#define _IRISGCCLASS_

#include "IrisDevHeader.h"
#include "IrisGC.h"

class IrisGCModule : public IIrisModule
{
public:
	static IrisValue ForceStart(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue SetFlag(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

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
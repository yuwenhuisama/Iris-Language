#ifndef _H_IRISDUMYMODULE_
#define _H_IRISDUMYMODULE_

#include "IrisInterfaces/IIrisModule.h"
class IrisDummyModule :
	public IIrisModule
{
public:

	const char* NativeModuleNameDefine() const {
		return nullptr;
	}

	IIrisModule* NativeUpperModuleDefine() const {
		return nullptr;
	}

	void NativeModuleDefine() {
	};

	IrisDummyModule();
	~IrisDummyModule();
};

#endif
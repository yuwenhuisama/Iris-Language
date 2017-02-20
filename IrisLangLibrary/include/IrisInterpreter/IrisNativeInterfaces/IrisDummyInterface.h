#ifndef _H_IRISDUMMYINTERFACE_
#define _H_IRISDUMMYINTERFACE_

#include "IrisInterfaces/IIrisInterface.h"

class IrisDummyInterface : public IIrisInterface
{
public:
	IrisDummyInterface();
	~IrisDummyInterface();

	// ͨ�� IIrisInterface �̳�
	virtual const char * NativeInterfaceNameDefine() const override;
	virtual IIrisModule * NativeUpperModuleDefine() const override;
	virtual void NativeInterfaceDefine() override;
};

#endif // !_H_IRISDUMMYINTERFACE_
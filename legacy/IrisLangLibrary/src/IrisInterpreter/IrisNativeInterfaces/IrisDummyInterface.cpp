#include "IrisInterpreter/IrisNativeInterfaces/IrisDummyInterface.h"



IrisDummyInterface::IrisDummyInterface()
{
}

IrisDummyInterface::~IrisDummyInterface()
{
}

const char * IrisDummyInterface::NativeInterfaceNameDefine() const
{
	return nullptr;
}

IIrisModule * IrisDummyInterface::NativeUpperModuleDefine() const
{
	return nullptr;
}

void IrisDummyInterface::NativeInterfaceDefine()
{
}

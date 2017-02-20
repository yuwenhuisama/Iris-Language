#ifndef _H_IIRISINTERFACE_
#define _H_IIRISINTERFACE_

class IrisInterface;
class IIrisModule;
class IIrisInterface
{
private:
	IrisInterface* m_pInternInterface = nullptr;

public:

	inline virtual IrisInterface* GetInternInterface() { return m_pInternInterface; }

	IIrisInterface() {}

	virtual ~IIrisInterface() = 0 {};

	virtual const char* NativeInterfaceNameDefine() const = 0;
	virtual IIrisModule* NativeUpperModuleDefine() const = 0;
	virtual void NativeInterfaceDefine() = 0;

	friend class IrisInterface;
	friend class IrisInterpreter;
};

#endif
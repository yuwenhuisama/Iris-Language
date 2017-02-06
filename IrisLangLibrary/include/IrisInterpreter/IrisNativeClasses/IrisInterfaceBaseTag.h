#ifndef _H_IRISINTERFACEBASETAG_
#define _H_IRISINTERFACEBASETAG_

#include "IrisDevHeader.h"

#if IR_USE_MEM_POOL
class IrisInterfaceBaseTag : public IrisObjectMemoryPoolInterface<IrisInterfaceBaseTag, POOLID_IrisInterfaceBaseTag>
#else
class IrisInterfaceBaseTag
#endif

{
private:
	IrisInterface* m_pInterface = nullptr;

public:

	const string& GetInterfaceName();
	void SetInterface(IrisInterface* pInterface);
	IrisInterface* GetInterface();

public:
	IrisInterfaceBaseTag(IrisInterface* pInterface);
	~IrisInterfaceBaseTag();
};

#endif
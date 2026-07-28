#ifndef _H_IRISMODULEBASETAG_
#define _H_IRISMODULEBASETAG_

#include "IrisDevHeader.h"

#if IR_USE_MEM_POOL
class IrisModuleBaseTag : public IrisObjectMemoryPoolInterface<IrisModuleBaseTag, POOLID_IrisModuleBaseTag>
#else
class IrisModuleBaseTag
#endif
{
private:
	IrisModule* m_pModule = nullptr;

public:
	IrisModuleBaseTag(IrisModule* pModule);

	void SetModule(IrisModule* pModule);
	const string& GetModuleName();

	IrisModule* GetModule();

	~IrisModuleBaseTag();
};

#endif
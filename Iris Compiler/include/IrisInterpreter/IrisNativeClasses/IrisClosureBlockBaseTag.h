#ifndef _H_IRISCLOSUREBLOCKBASETAG_
#define _H_IRISCLOSUREBLOCKBASETAG_

#include "IrisDevHeader.h"

class IrisClosureBlock;
#ifdef IR_USE_MEM_POOL
class IrisClosureBlockBaseTag : public IrisObjectMemoryPoolInterface<IrisClosureBlockBaseTag, POOLID_IrisClosureBlockBaseTag>
#else
class IrisClosureBlockBaseTag
#endif
{
private:
	IrisClosureBlock* m_pClosureBlock = nullptr;

public:

	IrisClosureBlock* GetClosureBlock();
	void SetClosureBlock(IrisClosureBlock* pColosureBlock);

	IrisClosureBlockBaseTag(IrisClosureBlock* pColosureBlock);
	~IrisClosureBlockBaseTag();
};

#endif
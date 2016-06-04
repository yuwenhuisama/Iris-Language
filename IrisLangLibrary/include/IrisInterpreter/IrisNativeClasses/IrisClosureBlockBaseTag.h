#ifndef _H_IRISCLOSUREBLOCKBASETAG_
#define _H_IRISCLOSUREBLOCKBASETAG_

#include "IrisDevHeader.h"

class IIrisClosureBlock;
#ifdef IR_USE_MEM_POOL
class IrisClosureBlockBaseTag : public IrisObjectMemoryPoolInterface<IrisClosureBlockBaseTag, POOLID_IrisClosureBlockBaseTag>
#else
class IrisClosureBlockBaseTag
#endif
{
private:
	IIrisClosureBlock* m_pClosureBlock = nullptr;

public:

	IIrisClosureBlock* GetClosureBlock();
	void SetClosureBlock(IIrisClosureBlock* pColosureBlock);

	IrisClosureBlockBaseTag(IIrisClosureBlock* pColosureBlock);
	~IrisClosureBlockBaseTag();
};

#endif
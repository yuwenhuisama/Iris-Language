#ifndef _H_IRISWITCHBLOCK_
#define _H_IRISWITCHBLOCK_
#include "IrisUnil/IrisList.h"

class IrisWhen;
class IrisBlock;
class IrisSwitchBlock
{
public:
	IrisList<IrisWhen*>* m_pWhenList = nullptr;
	IrisBlock* m_pElseBlock = nullptr;

public:
	IrisSwitchBlock(IrisList<IrisWhen*>* pWhenList, IrisBlock* pElseBlock);
	~IrisSwitchBlock();
};

#endif
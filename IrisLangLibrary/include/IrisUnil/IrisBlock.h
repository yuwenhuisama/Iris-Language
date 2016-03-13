#ifndef _H_IRISBLOCK_
#define _H_IRISBLOCK_

#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisList.h"

class IrisBlock
{
private:
	IrisList<IrisStatement*>* m_pStatements = nullptr;

public:
	IrisBlock(IrisList<IrisStatement*>* pStatements);

	bool Generate();

	~IrisBlock();
};

#endif
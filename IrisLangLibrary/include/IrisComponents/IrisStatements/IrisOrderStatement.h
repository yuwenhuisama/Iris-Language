#ifndef _H_IRISORDERSTATEMENT_
#define _H_IRISORDERSTATEMENT_
#include "IrisStatement.h"

class IrisBlock;
class IrisIdentifier;
class IrisOrderStatement :
	public IrisStatement
{
protected:
	IrisBlock* m_pOrderBlock = nullptr;
	IrisIdentifier* m_pIrregularObject = nullptr;
	IrisBlock* m_pServeBlock = nullptr;
	IrisBlock* m_pIgnoreBlock = nullptr;

public:

	virtual bool Generate();

	IrisOrderStatement(IrisBlock* pOrderBlock, IrisIdentifier* pIrregularObject, IrisBlock* pServeBlock, IrisBlock* pIgnoreBlock);
	virtual ~IrisOrderStatement();

	virtual bool Validate() override;
};

#endif
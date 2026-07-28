#ifndef _H_IRISFORSTATEMENT_
#define _H_IRISFORSTATEMENT_
#include "IrisStatement.h"

class IrisIdentifier;
class IrisExpression;
class IrisBlock;
class IrisForStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pIter1 = nullptr;
	IrisIdentifier* m_pIter2 = nullptr;
	IrisExpression* m_pSource = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisForStatement(IrisIdentifier* pIter1, IrisIdentifier* pIter2, IrisExpression* pSource, IrisBlock* pBlock);
	virtual ~IrisForStatement();

	virtual bool Validate() override;
};

#endif
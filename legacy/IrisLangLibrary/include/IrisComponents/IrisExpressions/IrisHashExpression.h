#ifndef _H_IRISHASHEXPRESSION_
#define _H_IRISHASHEXPRESSION_
#include "IrisExpression.h"
#include "IrisUnil/IrisList.h"

class IrisHashPair;
class IrisHashExpression :
	public IrisExpression
{
protected:
	IrisList<IrisHashPair*>* m_pHashPairs = nullptr;

public:

	virtual bool Generate();

	IrisHashExpression(IrisList<IrisHashPair*>* pHashPairs);
	virtual ~IrisHashExpression();

	virtual bool Validate() override;
};

#endif
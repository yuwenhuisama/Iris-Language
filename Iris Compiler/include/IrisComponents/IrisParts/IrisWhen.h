#ifndef _H_IRISWHEN_
#define _H_IRISWHEN_
#include "IrisUnil/IrisList.h"

class IrisExpression;
class IrisWhen
{
public:
	IrisList<IrisExpression*>* m_pExpressions = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:
	IrisWhen(IrisList<IrisExpression*>* pExpressions, IrisBlock* pBlock);
	~IrisWhen();
};

#endif
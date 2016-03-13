#ifndef _H_IRISELSEIF_
#define _H_IRISELSEIF_

class IrisExpression;
class IrisBlock;
class IrisElseIf
{
public:
	IrisExpression* m_pCondition = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:
	IrisElseIf(IrisExpression* pCondition, IrisBlock* pBlock);
	~IrisElseIf();
};

#endif
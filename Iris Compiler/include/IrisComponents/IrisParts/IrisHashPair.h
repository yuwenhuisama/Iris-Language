#ifndef _H_IRISHASHPAIR_
#define _H_IRISHASHPAIR_

class IrisExpression;
class IrisHashPair
{
public:
	IrisExpression* m_pKey = nullptr;
	IrisExpression* m_pValue = nullptr;

public:
	IrisHashPair(IrisExpression* pKey, IrisExpression* pValue);
	~IrisHashPair();
};

#endif
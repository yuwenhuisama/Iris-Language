#include "IrisComponents/IrisParts/IrisWhen.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"


IrisWhen::IrisWhen(IrisList<IrisExpression*>* pExpressions, IrisBlock* pBlock) : m_pExpressions(pExpressions), m_pBlock(pBlock)
{
}


IrisWhen::~IrisWhen()
{
	if (m_pExpressions) {
		m_pExpressions->Ergodic([](IrisExpression*& x) -> bool { delete x; x = nullptr; return true; });
		m_pExpressions->Clear();
		delete m_pExpressions;
	}
	if (m_pBlock)
		delete m_pBlock;
}

#include "IrisComponents/IrisParts/IrisElseIf.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisBlock.h"


IrisElseIf::IrisElseIf(IrisExpression* pCondition, IrisBlock* pBlock) : m_pCondition(pCondition), m_pBlock(pBlock)
{
}


IrisElseIf::~IrisElseIf()
{
	if (m_pCondition)
		delete m_pCondition;
	if (m_pBlock)
		delete m_pBlock;
}

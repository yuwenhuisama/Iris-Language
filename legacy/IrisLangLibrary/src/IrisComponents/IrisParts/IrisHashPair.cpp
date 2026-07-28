#include "IrisComponents/IrisParts/IrisHashPair.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"


IrisHashPair::IrisHashPair(IrisExpression* pKey, IrisExpression* pValue) : m_pKey(pKey), m_pValue(pValue)
{
}


IrisHashPair::~IrisHashPair()
{
	if (m_pKey)
		delete m_pKey;
	if (m_pValue)
		delete m_pValue;
}

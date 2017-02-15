#include "IrisUnil/IrisIdentifier.h"


IrisIdentifier::IrisIdentifier(IrisIdentifierType eType, char* szIdentifier) : m_eType(eType), m_strIdentifier(szIdentifier)
{
}

IrisIdentifier::~IrisIdentifier()
{
}

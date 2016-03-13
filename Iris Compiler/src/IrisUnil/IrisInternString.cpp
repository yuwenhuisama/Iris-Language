#include "IrisUnil/IrisInternString.h"



IrisInternString::IrisInternString(const string & strInterString) : m_strString(strInterString)
{
}

IrisInternString::IrisInternString(const char * szInterString) : m_strString(szInterString)
{
}

IrisInternString::~IrisInternString()
{
}

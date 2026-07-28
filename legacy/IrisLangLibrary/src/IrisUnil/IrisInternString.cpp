#include "IrisUnil/IrisInternString.h"



IrisInternString::IrisInternString(const string & strInterString) : m_strString(strInterString)
{
}

IrisInternString::IrisInternString(const char * szInternString) : m_strString(szInternString)
{
}

const char * IrisInternString::GetCTypeString() const {
	return m_strString.c_str();
}
const string & IrisInternString::GetSTLString() const {
	return m_strString;
}

void IrisInternString::SetHashed(bool bFlag) {
	m_bHashed = true;
}

bool IrisInternString::IsHashed() const { 
	return m_bHashed;
}

void IrisInternString::SetHashValue(size_t nHashValue) { 
	m_nHash = nHashValue; 
}

size_t IrisInternString::GetHashValue() const {
	return m_nHash;
}

IrisInternString::~IrisInternString()
{
}

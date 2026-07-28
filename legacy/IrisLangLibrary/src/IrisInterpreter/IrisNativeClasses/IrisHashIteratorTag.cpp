#include "IrisInterpreter/IrisNativeClasses/IrisHashIteratorTag.h"



void IrisHashIteratorTag::Initialize(IrisHashTag::_IrisHash* pHash)
{
	m_pHash = pHash;
	m_iIter = m_pHash->begin();
}

void IrisHashIteratorTag::Next()
{
	++m_iIter;
}

bool IrisHashIteratorTag::IsEnd()
{
	return m_iIter == m_pHash->end();
}

const IrisValue IrisHashIteratorTag::GetKey()
{
	return m_iIter->first;
}

const IrisValue IrisHashIteratorTag::GetValue()
{
	return m_iIter->second;
}

const IrisHashTag::_IrisHash::iterator & IrisHashIteratorTag::GetIter()
{
	return m_iIter;
}

IrisHashIteratorTag::IrisHashIteratorTag()
{
}


IrisHashIteratorTag::~IrisHashIteratorTag()
{
}

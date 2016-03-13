#include "IrisInterpreter/IrisNativeClasses/IrisHashTag.h"
#include "IrisDevelopUtil.h"


IrisHashTag::IrisHashTag() : m_mpHash()
{
}

void IrisHashTag::Initialize(IrisValues* pValues) {
	if (pValues) {
		int nSize = pValues->GetSize() / 2;
		for (int i = 0; i < nSize; ++i) {
			m_mpHash[(*pValues)[i * 2]] = (*pValues)[i * 2 + 1];
		}
	}
}

IrisValue IrisHashTag::At(const IrisValue& ivIndex) {
	if(m_mpHash.find(ivIndex) == m_mpHash.end()) {
		m_mpHash[ivIndex] = IrisDevUtil::Nil();
		return IrisDevUtil::Nil();
	}
	return m_mpHash[ivIndex];
}

IrisValue IrisHashTag::Set(const IrisValue& ivKey, const IrisValue& ivValue) {
	m_mpHash[ivKey] = ivValue;
	return ivValue;
}

int IrisHashTag::Size()
{
	return m_mpHash.size();
}

IrisHashTag::~IrisHashTag()
{
}

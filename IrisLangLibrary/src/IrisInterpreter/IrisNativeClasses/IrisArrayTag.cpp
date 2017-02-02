#include "IrisInterpreter/IrisNativeClasses/IrisArrayTag.h"
#include "IrisDevelopUtil.h"



IrisArrayTag::IrisArrayTag() : m_vcValues()
{
}

void IrisArrayTag::Initialize(IrisValues* pValues) {
	auto& vcValues = pValues->GetVector();
	if (pValues) {
		m_vcValues.assign(vcValues.begin(), vcValues.end());
	}
	else {
		m_vcValues.resize(0);
	}
}

IrisValue IrisArrayTag::At(int nIndex) {
	if (nIndex < 0) {
		return m_vcValues[m_vcValues.size() - (-nIndex % m_vcValues.size())];
	}
	else {
		if ((size_t)nIndex >= m_vcValues.size()) {
			for (size_t i = 0; i < nIndex - m_vcValues.size(); ++i) {
				m_vcValues.push_back(IrisDevUtil::Nil());
			}
			return IrisDevUtil::Nil();
		}
		else {
			return m_vcValues[nIndex];
		}
	}
}

IrisValue IrisArrayTag::Set(int nIndex, const IrisValue& ivValue) {
	if (nIndex < 0) {
		m_vcValues[m_vcValues.size() - (-nIndex % m_vcValues.size())] = ivValue;
	}
	else {
		if ((size_t)nIndex >= m_vcValues.size()) {
			for (size_t i = 0; i < nIndex - m_vcValues.size(); ++i) {
				m_vcValues.push_back(IrisDevUtil::Nil());
			}
			m_vcValues.push_back(ivValue);
		}
		else {
			m_vcValues[nIndex] = ivValue;
		}
	}
	return ivValue;
}

IrisValue IrisArrayTag::Push(const IrisValue& ivValue) {
	m_vcValues.push_back(ivValue);
	return ivValue;
}

IrisValue IrisArrayTag::Pop() {
	IrisValue ivValue = m_vcValues.back();
	m_vcValues.pop_back();
	return ivValue;
}

int IrisArrayTag::Size() {
	return m_vcValues.size();
}

bool IrisArrayTag::Empty() {
	return m_vcValues.empty();
}

int IrisArrayTag::IndexOf(IrisValue & pRef) {
	int i = 0;
	for (auto element : this->m_vcValues) {
		if (pRef == element) return i;
		i++;
	}
	return -1;
}

bool IrisArrayTag::Include(IrisValue& pRef) {
	for (auto element : this->m_vcValues) {
		if (pRef == element) return true;
	}
	return false;
}

IrisArrayTag & IrisArrayTag::Merge(IrisArrayTag& aRef) {
	m_vcValues.reserve(m_vcValues.size() + aRef.m_vcValues.size());
	m_vcValues.insert(m_vcValues.end(), aRef.m_vcValues.begin(), aRef.m_vcValues.end());
	return *this;
}

IrisArrayTag::~IrisArrayTag()
{
}

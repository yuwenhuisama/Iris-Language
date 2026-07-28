#include "IrisUnil/IrisValues.h"



IrisValues::IrisValues(const initializer_list<IrisValue>& ilValuesList) : m_vcValues(ilValuesList)
{
}

IrisValues::IrisValues() : m_vcValues()
{
}

IrisValues::IrisValues(size_t nSize) : m_vcValues(nSize)
{
}

void IrisValues::Add(const IrisValue & ivValue)
{
	m_vcValues.push_back(ivValue);
}

//bool IrisValues::Ergodic(const function<bool(IrisValue&)>& fErgodicFunction) {
//	for (auto& var : m_vcValues) {
//		if (!fErgodicFunction(var)) {
//			return false;
//		}
//	}
//	return true;
//}

IrisValues::~IrisValues()
{
}

#include "IrisInterpreter/IrisNativeClasses/IrisRangeIteratorTag.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisDevelopUtil.h"
#include <string>
#include <sstream>
using namespace std;


void IrisRangeIteratorTag::Initialize(IrisRangeTag * pRange)
{
	m_pRange = pRange;
	switch (pRange->m_eIterType)
	{
	case IrisRangeTag::_RangeIterType::Char:
		m_uIter.m_cChar = pRange->m_uFromData.m_cFrom;
		break;
	case IrisRangeTag::_RangeIterType::Integer:
		m_uIter.m_nInteger = pRange->m_uFromData.m_nFrom;
		break;
	default:
		break;
	}
}

void IrisRangeIteratorTag::Next()
{
	switch (m_pRange->m_eIterType)
	{
	case IrisRangeTag::_RangeIterType::Char:
		if (m_pRange->m_eIterDirection == IrisRangeTag::_RangeIterDirection::Plus) {
			++m_uIter.m_cChar;
		}
		else {
			--m_uIter.m_cChar;
		}
		break;
	case IrisRangeTag::_RangeIterType::Integer:
		if (m_pRange->m_eIterDirection == IrisRangeTag::_RangeIterDirection::Plus) {
			++m_uIter.m_nInteger;
		}
		else {
			--m_uIter.m_nInteger;
		}
		break;
	default:
		break;
	}
}

bool IrisRangeIteratorTag::IsEnd()
{
	switch (m_pRange->m_eIterType)
	{
	case IrisRangeTag::_RangeIterType::Char:
		if (m_pRange->m_eIterDirection == IrisRangeTag::_RangeIterDirection::Plus) {
			return m_uIter.m_cChar > m_pRange->m_uToData.m_cTo;
		}
		else {
			return m_uIter.m_cChar < m_pRange->m_uToData.m_cTo;
		}
		break;
	case IrisRangeTag::_RangeIterType::Integer:
		if (m_pRange->m_eIterDirection == IrisRangeTag::_RangeIterDirection::Plus) {
			return m_uIter.m_nInteger > m_pRange->m_uToData.m_nTo;
		}
		else {
			return m_uIter.m_nInteger < m_pRange->m_uToData.m_nTo;
		}
		break;
	default:
		break;
	}
	return false;
}

const IrisValue IrisRangeIteratorTag::GetValue()
{
	auto pInterpreter = IrisInterpreter::CurrentInterpreter();
	switch (m_pRange->m_eIterType)
	{
	case IrisRangeTag::_RangeIterType::Char:
	{
		stringstream sstream;
		sstream << m_uIter.m_cChar;
		return IrisDev_CreateInstanceByInstantValue((char*)sstream.str().c_str());
	}
		break;
	case IrisRangeTag::_RangeIterType::Integer:
		//return pInterpreter->GetIrisClass("Integer")->CreateInstanceByInstantValue(m_uIter.m_nInteger);
		return IrisDev_CreateInstanceByInstantValue(m_uIter.m_nInteger);
		break;
	default:
		break;
	}
	return pInterpreter->Nil();
}

IrisRangeIteratorTag::IrisRangeIteratorTag()
{
}


IrisRangeIteratorTag::~IrisRangeIteratorTag()
{
}

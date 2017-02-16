#include "IrisInterpreter/IrisNativeClasses/IrisRangeTag.h"



void IrisRangeTag::Initialize(IrisRangeType eType, char cFrom, char cTo)
{
	m_eType = eType;
	m_eIterType = _RangeIterType::Char;
	m_eIterDirection = cFrom > cTo ? _RangeIterDirection::Minus : _RangeIterDirection::Plus;
	m_uFromData.m_cFrom = cFrom;
	m_uToData.m_cTo = cTo;
}

void IrisRangeTag::Initialize(IrisRangeType eType, int nFrom, int nTo)
{
	m_eType = eType;
	m_eIterType = _RangeIterType::Integer;
	m_eIterDirection = nFrom > nTo ? _RangeIterDirection::Minus : _RangeIterDirection::Plus;

	switch (m_eType)
	{
	case IrisRangeType::LeftOpenAndRightOpen:
		switch (m_eIterDirection)
		{
		case IrisRangeTag::_RangeIterDirection::Plus:
			m_uFromData.m_nFrom = nFrom + 1;
			m_uToData.m_nTo = nTo - 1;
			break;
		case IrisRangeTag::_RangeIterDirection::Minus:
			m_uFromData.m_nFrom = nFrom - 1;
			m_uToData.m_nTo = nTo + 1;
			break;
		default:
			break;
		}
		break;
	case IrisRangeType::LeftOpenAndRightClosed:
		switch (m_eIterDirection)
		{
		case IrisRangeTag::_RangeIterDirection::Plus:
			m_uFromData.m_nFrom = nFrom + 1;
			m_uToData.m_nTo = nTo;
			break;
		case IrisRangeTag::_RangeIterDirection::Minus:
			m_uFromData.m_nFrom = nFrom - 1;
			m_uToData.m_nTo = nTo;
			break;
		default:
			break;
		}
		break;
	case IrisRangeType::LeftClosedAndRightClosed:
		m_uFromData.m_nFrom = nFrom;
		m_uToData.m_nTo = nTo;
		break;
	case IrisRangeType::LeftClosedAndRightOpen:
		switch (m_eIterDirection)
		{
		case IrisRangeTag::_RangeIterDirection::Plus:
			m_uFromData.m_nFrom = nFrom;
			m_uToData.m_nTo = nTo - 1;
			break;
		case IrisRangeTag::_RangeIterDirection::Minus:
			m_uFromData.m_nFrom = nFrom;
			m_uToData.m_nTo = nTo + 1;
			break;
		default:
			break;
		}
		break;
	default:
		break;
	}
}

IrisRangeTag::IrisRangeTag()
{
}


IrisRangeTag::~IrisRangeTag()
{
}

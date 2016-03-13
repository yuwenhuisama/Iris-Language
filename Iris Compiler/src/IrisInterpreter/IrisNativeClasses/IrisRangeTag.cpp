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
	m_uFromData.m_nFrom = nFrom;
	m_uToData.m_nTo = nTo;
}

IrisRangeTag::IrisRangeTag()
{
}


IrisRangeTag::~IrisRangeTag()
{
}

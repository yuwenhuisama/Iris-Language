#ifndef _H_IRISRANGETAGE_
#define _H_IRISRANGETAGE_

#include "IrisComponents/IrisComponentsDefines.h"
#include "IrisDevHeader.h"

#ifdef IR_USE_MEM_POOL
class IrisRangeTag : public IrisObjectMemoryPoolInterface<IrisRangeTag, POOLID_IrisRangeTag>
#else
class IrisRangeTag
#endif
{
private:
	enum class _RangeIterType {
		Char = 0,
		Integer,
	};

	enum class _RangeIterDirection {
		Plus = 0,
		Minus,
	};

	IrisRangeType m_eType;
	_RangeIterType m_eIterType = _RangeIterType::Char;
	_RangeIterDirection m_eIterDirection = _RangeIterDirection::Plus;

private:
	union {
		char m_cFrom;
		int m_nFrom;
	} m_uFromData;

	union {
		char m_cTo;
		int m_nTo;
	} m_uToData;

public:

	void Initialize(IrisRangeType eType, char cFrom, char cTo);
	void Initialize(IrisRangeType eType, int nFrom, int nTo);

	IrisRangeTag();
	~IrisRangeTag();

	friend class IrisRangeIteratorTag;
};

#endif
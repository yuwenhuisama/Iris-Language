#ifndef _H_IRISRANGEITERATORTAG_
#define _H_IRISRANGEITERATORTAG_

#include "IrisDevHeader.h"
#include "IrisRangeTag.h"

class IrisRangeIteratorTag
{
private:
	union Iter {
		char m_cChar;
		int m_nInteger;
	} m_uIter;

	IrisRangeTag* m_pRange = nullptr;

public:

	void Initialize(IrisRangeTag* pRange);
	void Next();
	bool IsEnd();
	const IrisValue GetValue();

	IrisRangeIteratorTag();
	~IrisRangeIteratorTag();
};

#endif
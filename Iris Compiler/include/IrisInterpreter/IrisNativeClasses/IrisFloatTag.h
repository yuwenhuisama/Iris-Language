#ifndef _H_IRISFLOATTAG_
#define _H_IRISFLOATTAG_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"

#include <math.h>
#include <sstream>

#ifdef IR_USE_MEM_POOL
class IrisFloatTag : public IrisObjectMemoryPoolInterface<IrisFloatTag, POOLID_IrisFloatTag>
#else
class IrisFloatTag
#endif
{
private:
	double m_dFloat;

public:
	IrisFloatTag();
	IrisFloatTag(char* szLiterral);
	IrisFloatTag(double dFloat);

	//IrisFloatTag(IrisIntegerTag& iiInteger) {
	//	m_dFloat = (double)iiInteger.m_nInteger;
	//}

	operator IrisIntegerTag() {
		IrisIntegerTag iitInteger;
		iitInteger.m_nInteger = (int)m_dFloat;
		return iitInteger;
	}

	IrisFloatTag Add(IrisFloatTag& iftFloat);
	IrisFloatTag Sub(IrisFloatTag& iftFloat);
	IrisFloatTag Mul(IrisFloatTag& iftFloat);
	IrisFloatTag Div(IrisFloatTag& iftFloat);
	IrisFloatTag Power(IrisFloatTag& iftFloat);

	bool Equal(IrisFloatTag& iftFloat);
	bool NotEqual(IrisFloatTag& iftFloat);

	bool BigThan(IrisFloatTag& iftFloat);
	bool BigThanOrEqual(IrisFloatTag& iftFloat);
	bool LessThan(IrisFloatTag& iftFloat);
	bool LessThanOrEqual(IrisFloatTag& iftFloat);

	IrisFloatTag Plus();
	IrisFloatTag Minus();

	string ToString();

	~IrisFloatTag();

	friend class IrisIntegerTag;
	friend class IrisFloat;
};

#endif
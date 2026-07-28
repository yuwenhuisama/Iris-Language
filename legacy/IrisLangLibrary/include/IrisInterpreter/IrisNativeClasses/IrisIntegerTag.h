#ifndef _H_IRISINTEGERTAG_
#define _H_IRISINTEGERTAG_

class IrisFloatTag;

#include "IrisDevHeader.h"

#include <math.h>
#include <sstream>
using namespace std;
#if IR_USE_MEM_POOL
class IrisIntegerTag : public IrisObjectMemoryPoolInterface<IrisIntegerTag, POOLID_IrisIntegerTag>
#else
class IrisIntegerTag
#endif
{
private:
	int m_nInteger;

public:
	IrisIntegerTag();
	IrisIntegerTag(char* szLiterral);
	IrisIntegerTag(int nInteger);

	IrisIntegerTag Add(IrisIntegerTag& iitInteger);
	IrisIntegerTag Sub(IrisIntegerTag& iitInteger);
	IrisIntegerTag Mul(IrisIntegerTag& iitInteger);
	IrisIntegerTag Div(IrisIntegerTag& iitInteger);
	IrisIntegerTag Mod(IrisIntegerTag& iitInteger);
	IrisIntegerTag Power(IrisIntegerTag& iitInteger);

	IrisIntegerTag Shr(IrisIntegerTag& iitInteger);
	IrisIntegerTag Shl(IrisIntegerTag& iitInteger);
	IrisIntegerTag Sar(IrisIntegerTag& iitInteger);
	IrisIntegerTag Sal(IrisIntegerTag& iitInteger);
	IrisIntegerTag BitXor(IrisIntegerTag& iitInteger);
	IrisIntegerTag BitAnd(IrisIntegerTag& iitInteger);
	IrisIntegerTag BitOr(IrisIntegerTag& iitInteger);
	
	IrisIntegerTag BitNot();

	bool Equal(IrisIntegerTag& iitInteger);
	bool NotEqual(IrisIntegerTag& iitInteger);

	bool BigThan(IrisIntegerTag& iitInteger);
	bool BigThanOrEqual(IrisIntegerTag& iitInteger);
	bool LessThan(IrisIntegerTag& iitInteger);
	bool LessThanOrEqual(IrisIntegerTag& iitInteger);

	IrisIntegerTag Plus();
	IrisIntegerTag Minus();

	operator IrisFloatTag();

	string ToString();

	inline void SetInteger(int nInteger) { m_nInteger = nInteger; };
	inline int GetInteger() { return m_nInteger; }

	~IrisIntegerTag();

	friend class IrisFloatTag;
	friend class IrisInteger;
	friend class IrisStatement;
	friend class IrisInterpreter;
};

#endif
#ifndef _H_IRISARRAYTAG_
#define _H_IRISARRAYTAG_

#include "IrisDevHeader.h"

#include <vector>
using namespace std;

#ifdef IR_USE_MEM_POOL
class IrisArrayTag : public IrisObjectMemoryPoolInterface<IrisArrayTag, POOLID_IrisArrayTag>
#else
class IrisArrayTag
#endif
{
private:
	vector<IrisValue> m_vcValues;

public:

	void Initialize(IrisValues* pValues);
	IrisValue At(int nIndex);
	IrisValue Set(int nIndex, const IrisValue& ivValue);
	IrisValue Push(const IrisValue& ivValue);
	IrisValue Pop();
	int Size();

	IrisArrayTag();
	~IrisArrayTag();

	friend class IrisArray;
	friend class IrisArrayIterator;
	friend class IrisStatement;
};

#endif
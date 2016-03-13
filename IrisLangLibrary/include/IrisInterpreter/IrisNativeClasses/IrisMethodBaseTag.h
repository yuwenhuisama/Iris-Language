#ifndef _IRIS_METHODBASETAG_
#define _IRIS_METHODBASETAG_
#include "IrisDevHeader.h"

class IrisMethod;
#ifdef IR_USE_MEM_POOL
class IrisMethodBaseTag : public IrisObjectMemoryPoolInterface<IrisMethodBaseTag, POOLID_IrisMethodBaseTag>
#else
class IrisMethodBaseTag
#endif
{
private:
	IrisMethod* m_pMethod = nullptr;

public:
	const string& GetMethodName();
	void SetMethod(IrisMethod* pMethod);

	IrisMethod* GetMethod();

	IrisMethodBaseTag(IrisMethod* pMethod);
	~IrisMethodBaseTag();
};

#endif
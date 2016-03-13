#ifndef _H_IRISCLASSBASETGA_
#define _H_IRISCLASSBASETGA_

#include "IrisDevHeader.h"

#ifdef IR_USE_MEM_POOL
class IrisClassBaseTag : public IrisObjectMemoryPoolInterface<IrisClassBaseTag, POOLID_IrisClassBaseTag>
#else
class IrisClassBaseTag
#endif
{
private:
	IrisClass* m_pClass = nullptr;

public:

	const string& GetThisClassName();
	void SetClass(IrisClass* pClass);
	IrisClass* GetClass();

	IrisClassBaseTag(IrisClass* pClass);
	~IrisClassBaseTag();
};

#endif
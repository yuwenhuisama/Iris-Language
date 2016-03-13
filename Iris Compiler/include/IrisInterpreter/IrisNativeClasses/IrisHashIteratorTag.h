#ifndef _H_IRISHASHITERATORTAG_
#define _H_IRISHASHITERATORTAG_

#include "IrisHashTag.h"

class IrisHashIteratorTag
{
private:
	IrisHashTag::_IrisHash::iterator m_iIter;
	IrisHashTag::_IrisHash* m_pHash = nullptr;

public:
	void Initialize(IrisHashTag::_IrisHash* pHash);
	void Next();
	bool IsEnd();
	const IrisValue GetKey();
	const IrisValue GetValue();
	const IrisHashTag::_IrisHash::iterator& GetIter();

public:
	IrisHashIteratorTag();
	~IrisHashIteratorTag();
};

#endif
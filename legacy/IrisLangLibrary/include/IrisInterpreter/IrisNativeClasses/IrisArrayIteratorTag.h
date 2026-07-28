#ifndef _IRIS_ARAYITERATORTAG_
#define _IRIS_ARAYITERATORTAG_

#include "IrisUnil/IrisValue.h"
#include <vector>
using namespace std;

class IrisArrayIteratorTag
{
private:
	typedef vector<IrisValue>::iterator Iterator;
	Iterator m_iIter;
	vector<IrisValue>* m_pArray = nullptr;

public:
	void Initialize(vector<IrisValue>* pArray);
	IrisArrayIteratorTag Next();
	bool IsEnd();
	const IrisValue GetValue();
	const vector<IrisValue>::iterator& GetIter();

	IrisArrayIteratorTag();
	~IrisArrayIteratorTag();
};

#endif
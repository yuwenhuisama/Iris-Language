#ifndef _H_IRISSTACK_
#define _H_IRISSTACK_

#include "IrisValue.h"
#include <vector>
using namespace std;

class IrisStack
{
private:
	vector<IrisValue> m_lsStack;

public:
	IrisValue Pop();
	void Push(IrisValue ivValue);
	void Clear();

	IrisStack();
	~IrisStack();

	friend class IrisGC;
	friend class IrisInterpreter;
};

#endif
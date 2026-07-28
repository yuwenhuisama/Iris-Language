#ifndef _H_IRISCONDITIONVARIABLETAG_
#define _H_IRISCONDITIONVARIABLETAG_

#include <thread>
#include <condition_variable>
#include <mutex>
using namespace std;

class IrisConditionVariableTag
{
private:
	condition_variable_any m_cvaConditionVariable;
	recursive_mutex* m_pMutex = nullptr;

public:

	void Initialize(recursive_mutex* pMutex);
	void NotifyOne();
	void NotifyAll();
	void Wait();

	IrisConditionVariableTag();
	~IrisConditionVariableTag();
};

#endif
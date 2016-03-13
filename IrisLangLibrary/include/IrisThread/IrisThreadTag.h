#ifndef _H_IRISTHREADTAG_
#define _H_IRISTHREADTAG_
#include <thread>
#include <condition_variable>
using namespace std;
class IrisThreadTag
{
private:
	thread* m_pThread = nullptr;
	mutex m_rmReadyMutex;

public:
	IrisThreadTag();
	void Initialize(thread* pThread);
	void Join();
	void Detach();
	size_t GetThreadId();

	thread* GetThread();
	mutex& GetMutex();

	~IrisThreadTag();
};

#endif
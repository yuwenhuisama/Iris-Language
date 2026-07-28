#ifndef _H_IRISMUTEXTTAG_
#define _H_IRISMUTEXTTAG_
#include <thread>
#include <mutex>
using namespace std;
class IrisMutexTag
{
private:
	recursive_mutex m_rmMutex;

public:

	void Lock();
	void Unlock();

	recursive_mutex& GetMutex();

	IrisMutexTag();
	~IrisMutexTag();
};

#endif
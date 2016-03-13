/* Thanks for : http://blog.csdn.net/raomeng1/article/details/7685421 */

#ifndef _H_IRISWLLOCK
#define _H_IRISWLLOCK
#include <thread>
#include <mutex>
#include <condition_variable>
using namespace std;

class IrisWLLock
{
public:
	IrisWLLock()
		: stat(0)
	{
	}

	void ReadLock()
	{
		mtx.lock();
		while (stat < 0)
			cond.wait(mtx);
		++stat;
		mtx.unlock();
	}

	void ReadUnlock()
	{
		mtx.lock();
		if (--stat == 0)
			cond.notify_one(); // 叫醒一个等待的写操作  
		mtx.unlock();
	}

	void WriteLock()
	{
		mtx.lock();
		while (stat != 0)
			cond.wait(mtx);
		stat = -1;
		mtx.unlock();
	}

	void WriteUnlock()
	{
		mtx.lock();
		stat = 0;
		cond.notify_all(); // 叫醒所有等待的读和写操作  
		mtx.unlock();
	}

private:
	mutex mtx;
	condition_variable_any cond;
	int stat; // == 0 无锁；> 0 已加读锁个数；< 0 已加写锁  
};

#endif
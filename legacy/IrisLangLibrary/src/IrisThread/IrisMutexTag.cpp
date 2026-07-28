#include "IrisThread/IrisMutexTag.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"


void IrisMutexTag::Lock()
{
	//IrisThreadManager::SetThreadBlock(this_thread::get_id(), true);
	//m_rmMutex.lock();
	//IrisThreadManager::SetThreadBlock(this_thread::get_id(), false);

	// Try
	bool bResult = m_rmMutex.try_lock();
	if (!bResult) {
		IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), true);
		if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
			IrisGC::CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
		}
		m_rmMutex.lock();
		IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), false);
	}
}

void IrisMutexTag::Unlock()
{
	m_rmMutex.unlock();
}

recursive_mutex&  IrisMutexTag::GetMutex()
{
	return m_rmMutex;
}

IrisMutexTag::IrisMutexTag()
{
}


IrisMutexTag::~IrisMutexTag()
{
}

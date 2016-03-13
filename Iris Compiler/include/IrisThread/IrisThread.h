#ifndef _H_IRISTHREAD_
#define _H_IRISTHREAD_

#include "IrisDevHeader.h"
#include "IrisInterpreter.h"
#include "IrisThread/IrisThreadTag.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"

class IrisThread : public IIrisClass
{
public:
	static void ThreadBlockRun(IrisClosureBlock* pBlock, IrisThreadTag* pNativeThreadObject, IrisObject* pThreadObject) {
		unique_lock<mutex> ulLock(pNativeThreadObject->GetMutex());
		pThreadObject->SetPermanent(true);
		thread::id nId = this_thread::get_id();
		IrisThreadManager::CurrentThreadManager()->CurrentThreadManager()->AddNewThread(nId, pNativeThreadObject->GetThread());
		IrisThreadManager::CurrentThreadManager()->CurrentThreadManager()->AddNewThreadInfo(nId, new IrisThreadUniqueInfo());
		IrisGC::CurrentGC()->AddThreadGCData(nId, new IrisGC::ThreadGCData());
		static_cast<IrisContextEnvironment*>(pBlock->GetCurrentContextEnvironment())->m_bIsThreadMainContext = true;

		pBlock->Excute(nullptr);

		// Ignore Irregular having happened in thread.
		if (IrisInterpreter::CurrentInterpreter()->IrregularHappened()) {
			IrisInterpreter::CurrentInterpreter()->UnregistIrregular();
		}

		static_cast<IrisContextEnvironment*>(pBlock->GetCurrentContextEnvironment())->m_bIsThreadMainContext = false;
		IrisGC::CurrentGC()->TransferCurrentGCDataToMainThread();
		IrisGC::CurrentGC()->RemoveThreadGCData(nId);
		IrisThreadManager::CurrentThreadManager()->TransferCurrentThreadHeapToMainThread();
		IrisThreadManager::CurrentThreadManager()->DeleteThreadInfo(nId);
		IrisThreadManager::CurrentThreadManager()->DeleteThread(nId);
		IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pBlock->GetCurrentContextEnvironment());
		while (pTmpEnv) {
			--pTmpEnv->m_nReferenced;
			pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
		}
		delete pBlock;
		pThreadObject->SetPermanent(false);

		if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
			IrisGC::CurrentGC()->CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
		}
	}

public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisContextEnvironment* pTmpContextEnvironment = static_cast<IrisContextEnvironment*>(pContextEnvironment);
		if (!pTmpContextEnvironment->m_pClosureBlock) {
			IrisDev_GroanIrregularWithString("A Thread Object needs a block to run.");
			return IrisDev_Nil();
		}
		
		auto pThreadTag = IrisDev_GetNativePointer<IrisThreadTag*>(ivObj);
		static_cast<IrisContextEnvironment*>(static_cast<IrisClosureBlock*>(pTmpContextEnvironment->GetClosureBlock())->GetCurrentContextEnvironment())->m_bIsThreadMainContext = true;
		unique_lock<mutex> ulLock(pThreadTag->GetMutex());
		auto pThread = new thread(ThreadBlockRun, static_cast<IrisClosureBlock*>(pContextEnvironment->GetClosureBlock()), pThreadTag, static_cast<IrisObject*>(ivObj.GetIrisObject()));
		pTmpContextEnvironment->m_pClosureBlock = nullptr;
		pThreadTag->Initialize(pThread);
		return ivObj;
	}

	static IrisValue Join(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pThreadTag = IrisDev_GetNativePointer<IrisThreadTag*>(ivObj);
		IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), true);
		if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
			IrisGC::CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
		}
		pThreadTag->Join();
		IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), false);
		return IrisDev_Nil();
	}

	static IrisValue Detach(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pThreadTag = IrisDev_GetNativePointer<IrisThreadTag*>(ivObj);
		pThreadTag->Detach();
		return IrisDev_Nil();
	}

	static IrisValue GetID(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pThreadTag = IrisDev_GetNativePointer<IrisThreadTag*>(ivObj);
		return IrisDev_CreateInt(pThreadTag->GetThreadId());
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Thread";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisThreadTag);
	}

	void* NativeAlloc() {
		return new IrisThreadTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisThreadTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "join", Join, 0, false);
		IrisDev_AddInstanceMethod(this, "detach", Detach, 0, false);
		IrisDev_AddGetter(this, "@id", GetID);
	}

	IrisThread();
	~IrisThread();
};

#endif
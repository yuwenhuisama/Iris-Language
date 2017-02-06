#ifndef _H_IRISINTERPRETER_
#define _H_IRISINTERPRETER_

#include "IrisCompileConfigure.h"

#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisUnil/IrisValue.h"
#include "IrisUnil/IrisTree.h"
#include "IrisUnil/IrisHeap.h"
#include "IrisUnil/IrisStack.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisDevelopUtil.h"
#include "IrisUnil/IrisCodeSegment.h"
#include "IrisThread/IrisWLLock.h"

#include "IrisUnil/IrisInternString.h"

#include <unordered_map>
#include <vector>
#include <list>
#include <unordered_set>
#include <thread>

#ifdef _MSC_VER
#include <Windows.h>
#endif

using namespace std;

class IrisObject;
class IrisInterface;
class IrisClass;
class IrisModule;
class IrisMethod;
class IrisContextEnvironment;
class IrisCompiler;
class IrisClosureBlock;

class IrisInterpreter
{
private:

#if IR_USE_STL_STRING
	typedef unordered_map<string, IrisValue> _ValueMap;
	typedef pair<string, IrisValue> _ValuePair;
	typedef unordered_map<string, IrisMethod*> _MethodMap;
	typedef pair<string, IrisMethod*> _MethodPair;
#else
	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _ValueMap;
	typedef pair<IrisInternString, IrisValue> _ValuePair;
	typedef unordered_map<IrisInternString, IrisMethod*, IrisInternString::IrisInerStringHash> _MethodMap;
	typedef pair<IrisInternString, IrisMethod*> _MethodPair;
#endif // IR_USE_STL_STRING
	typedef unordered_map<string, HMODULE> _ExtentionMap;
	typedef pair<string, HMODULE> _ExtentionPair;
	
	typedef bool (*ExitConditonFunction)();

public:

#define INNER_CLASS_MEMBER_NAME(klass) m_p##klass##Class
#define INNER_CLASS_MEMBER(klass) IIrisClass* INNER_CLASS_MEMBER_NAME(klass) = nullptr
#define INNER_CLASS_GET_POINTER(klass)  IrisInterpreter::CurrentInterpreter()->m_icrInnerClassesRecord.INNER_CLASS_MEMBER_NAME(klass)
#define INNER_CLASS_SET_POINTER(klass, p) IrisInterpreter::CurrentInterpreter()->m_icrInnerClassesRecord.INNER_CLASS_MEMBER_NAME(klass) = p
	struct InterClassesRecord {
		INNER_CLASS_MEMBER(Class);
		INNER_CLASS_MEMBER(Module);
		INNER_CLASS_MEMBER(Interface);
		INNER_CLASS_MEMBER(Object);
		INNER_CLASS_MEMBER(String);
		INNER_CLASS_MEMBER(UniqueString);
		INNER_CLASS_MEMBER(Integer);
		INNER_CLASS_MEMBER(Float);
		INNER_CLASS_MEMBER(Array);
		INNER_CLASS_MEMBER(Hash);
		INNER_CLASS_MEMBER(Range);
		INNER_CLASS_MEMBER(Block);
	};

public:
	InterClassesRecord m_icrInnerClassesRecord;

private:
	static IrisInterpreter* s_pInstance;

private:

	recursive_mutex m_rmHeapInsertMT;

	IrisHeap m_hpHeap;
	//IrisStack m_stStack;

	//IrisTree<IrisModule*> m_trModuels;
	//_ClassMap m_mpClasses;
	//_InterfaceMap m_mpInterfaces;
	_MethodMap m_mpMethods;

	_ValueMap m_mpGlobalValues;
	_ValueMap m_mpOtherValues;
	_ValueMap m_mpConstances;

	IrisValue m_ivNil;
	IrisValue m_ivTrue;
	IrisValue m_ivFalse;

	IrisCompiler* m_pCurrentCompiler = nullptr;

	ExitConditonFunction m_pfExitConditionFunction = nullptr;

	IrisWLLock m_iwlConstanceLock;
	IrisWLLock m_iwlGlobalVariableLock;
	IrisWLLock m_iwlOtherVariableLock;
	IrisWLLock m_iwlMethodLock;

	IrisWLLock m_iwlInterfaceLock;
	IrisWLLock m_iwlClassLock;
	IrisWLLock m_iwlModuleLock;

	_ExtentionMap m_emLoadedExtention;

private:
	IrisInterpreter();
#if IR_USE_STL_STRING
	int _Split(const string& str, list<string>& ret_, string sep = ",");
	IrisModule* _GetLastModuleFromPath(const list<string>& lsPath);
#else
	int _Split(const string& str, list<IrisInternString>& ret_, string sep = ",");
	IrisModule* _GetLastModuleFromPath(const list<IrisInternString>& lsPath);
#endif // IR_USE_STL_STRING

public:
	static IrisInterpreter* CurrentInterpreter();
	static void SetCurrentInterpreter(IrisInterpreter* pInter);

public:

	IrisAM GetOneAM(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);

	IrisClass* GetIrisClass(const string& strClassFullFieldName);
	IrisModule* GetIrisModule(const string& strModuleFullFiledName);
	IrisInterface* GetIrisInterface(const string& strInterfaceFullFiledName);

#if IR_USE_STL_STRING
	IrisClass* GetIrisClass(const list<string>& lsRoute);
	IrisModule* GetIrisModule(const list<string>& lsRoute);
	IrisInterface* GetIrisInterface(const list<string>& lsRoute);
#else
	IrisClass* GetIrisClass(const list<IrisInternString>& lsRoute);
	IrisModule* GetIrisModule(const list<IrisInternString>& lsRoute);
	IrisInterface* GetIrisInterface(const list<IrisInternString>& lsRoute);
#endif // IR_USE_STL_STRING

	bool RegistClass(const string& strClassFullFieldName, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(const string& strModuleFullFieldName, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(const string& strInterfaceFullFieldName, IIrisInterface* pInterface, bool bNative = true);

#if IR_USE_STL_STRING
	bool RegistClass(list<string>& lsPath, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(list<string>& lsPath, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(list<string>& lsPath, IIrisInterface* pInterface, bool bNative = true);
#else
	bool RegistClass(list<IrisInternString>& lsPath, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(list<IrisInternString>& lsPath, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(list<IrisInternString>& lsPath, IIrisInterface* pInterface, bool bNative = true);
#endif // IR_USE_STL_STRING

	bool AddNewInstanceToHeap(IrisValue& ivValue);
	bool AddNewEnvironmentToHeap(IrisContextEnvironment* pEnvironment);

	bool Initialize();
	bool ShutDown();

public:

	inline void SetCompiler(IrisCompiler* pCompiler) { m_pCurrentCompiler = pCompiler; };
	bool Run();
	//bool RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer);
	bool RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer);
	inline void SetExitConditionFunction(ExitConditonFunction pfFunction) { m_pfExitConditionFunction = pfFunction; }

	bool LoadExtension(const string& strExtensionPath);

	~IrisInterpreter();

	// inlines
public:


	//inline void RegisiterClosuerBlock(IrisClosureBlock* pBlock) {
	//	m_pClosureBlockRegister = pBlock;
	//}

	void PopStack(unsigned int nTimes);

	inline bool FatalErrorHappened() { return IrisDevUtil::GetCurrentThreadInfo()->m_bFatalErrorHappendRegister; }
	inline void HappenFatalError() { IrisDevUtil::GetCurrentThreadInfo()->m_bFatalErrorHappendRegister = true; }

	inline bool IrregularHappened() { return IrisDevUtil::GetCurrentThreadInfo()->m_bIrregularHappenedRegister; }
	inline void RegistIrregular(IrisValue& ivValue) {
		IrisDevUtil::GetCurrentThreadInfo()->m_ivIrregularObjectRegister = ivValue; 
		IrisDevUtil::GetCurrentThreadInfo()->m_bIrregularHappenedRegister = true;
	}
	inline void UnregistIrregular() { IrisDevUtil::GetCurrentThreadInfo()->m_bIrregularHappenedRegister = false; }

	inline bool ForceStop() { return IrisDevUtil::GetCurrentThreadInfo()->m_bIrregularHappenedRegister; }

	inline bool PushEnvironment() {
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_skEnvironmentStack.push_back(pInfo->m_pEnvrionmentRegister);
		return true;
	}
	inline bool SetEnvironment(IrisContextEnvironment* pEnvironment) {
		IrisDevUtil::GetCurrentThreadInfo()->m_pEnvrionmentRegister = pEnvironment;
		return true;
	}
	inline bool PopEnvironment() {
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_pEnvrionmentRegister = pInfo->m_skEnvironmentStack.back();
		pInfo->m_skEnvironmentStack.pop_back();
		return true;
	}

	inline void PushVessle() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_skVessleRegister.push_back(pInfo->m_ivVessleRegister); 
	}
	inline void PopVessle() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_ivVessleRegister = pInfo->m_skVessleRegister.back();
		pInfo->m_skVessleRegister.pop_back(); 
	}

	inline void PushIterator() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_skIteratorRegister.push_back(pInfo->m_ivIteratorRegister); }
	inline void PopIterator() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_ivIteratorRegister = pInfo->m_skIteratorRegister.back();
		pInfo->m_skIteratorRegister.pop_back();
	}

	inline void PushCounter() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_skCounterRegister.push_back(pInfo->m_ivCounterRegister);
	}
	inline void PopCounter() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_ivCounterRegister = pInfo->m_skCounterRegister.back(); 
		pInfo->m_skCounterRegister.pop_back(); 
	}

	inline void PushTimer() {
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_skTimerRegister.push_back(pInfo->m_ivTimerRegister); }
	inline void PopTimer() {
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_ivTimerRegister = pInfo->m_skTimerRegister.back();
		pInfo->m_skTimerRegister.pop_back();
	}

	inline void PushUnlimitedLoopFlag() {
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_skUnimitedLoopFlag.push_back(pInfo->m_bUnimitedLoopFlagRegister); }
	inline void PopUnlimitedLoopFlag() { 
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		pInfo->m_bUnimitedLoopFlagRegister = pInfo->m_skUnimitedLoopFlag.back();
		pInfo->m_skUnimitedLoopFlag.pop_back(); 
	}

	inline unsigned int GetTopDeepIndex() {
		auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		return pInfo->m_skDeepStack.empty() ? -1 : pInfo->m_skDeepStack.back(); 
	}
	inline void PushDeepIndex(unsigned int nIndex) { IrisDevUtil::GetCurrentThreadInfo()->m_skDeepStack.push_back(nIndex); }
	inline void PopTopDeepIndex() { IrisDevUtil::GetCurrentThreadInfo()->m_skDeepStack.pop_back(); }
	inline void ClearDeepStack() { IrisDevUtil::GetCurrentThreadInfo()->m_skDeepStack.clear(); }

	inline unsigned int GetTopMethodDeepIndex() { return IrisDevUtil::GetCurrentThreadInfo()->m_skMethodDeepStack.back(); }
	inline void PushMethodDeepIndex(unsigned int nIndex) { IrisDevUtil::GetCurrentThreadInfo()->m_skMethodDeepStack.push_back(nIndex); }
	inline void PopMethodTopDeepIndex() { IrisDevUtil::GetCurrentThreadInfo()->m_skMethodDeepStack.pop_back(); }
	inline void ClearMethodDeepStack() { IrisDevUtil::GetCurrentThreadInfo()->m_skMethodDeepStack.clear(); }

	inline const IrisValue& GetCurrentResultRegister() { return IrisDevUtil::GetCurrentThreadInfo()->m_ivResultRegister; }

	inline IrisContextEnvironment* GetCurrentContextEnvrionment() {
		return IrisDevUtil::GetCurrentThreadInfo()->m_pEnvrionmentRegister;
	}

#if IR_USE_STL_STRING
	inline void AddConstance(const string& strValueName, const IrisValue& ivValue) {
#else
	inline void AddConstance(const IrisInternString& strValueName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
		m_iwlConstanceLock.WriteLock();
		if (m_mpConstances.find(strValueName) != m_mpConstances.end()) {
			m_iwlConstanceLock.WriteUnlock();
			return;
		}
		m_mpConstances.insert(_ValuePair(strValueName, ivValue));
		m_iwlConstanceLock.WriteUnlock();
	}

#if IR_USE_STL_STRING
	inline void AddGlobalValue(const string& strValueName, const IrisValue& ivValue) {
#else
	inline void AddGlobalValue(const IrisInternString& strValueName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
		m_iwlGlobalVariableLock.WriteLock();
		m_mpGlobalValues.insert(_ValuePair(strValueName, ivValue));
		m_iwlGlobalVariableLock.WriteUnlock();
	}

#if IR_USE_STL_STRING
	inline void AddOtherValue(const string& strValueName, const IrisValue& ivValue) {
#else
	inline void AddOtherValue(const IrisInternString& strValueName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
		m_iwlOtherVariableLock.WriteLock();
		m_mpOtherValues.insert(_ValuePair(strValueName, ivValue));
		m_iwlOtherVariableLock.WriteUnlock();
	}

#if IR_USE_STL_STRING
	inline const IrisValue& GetConstance(const string& strValueName, bool& bResult) {
#else
	inline const IrisValue& GetConstance(const IrisInternString& strValueName, bool& bResult) {
#endif // IR_USE_STL_STRING
		bResult = true;
		m_iwlConstanceLock.ReadLock();
		decltype(m_mpConstances)::iterator iCons;
		if ((iCons = m_mpConstances.find(strValueName)) == m_mpConstances.end()) {
			bResult = false;
			m_iwlConstanceLock.ReadUnlock();
			return m_ivNil;
		}
		else {
			auto& ivResult = iCons->second;
			m_iwlConstanceLock.ReadUnlock();
			return ivResult;
		}
	}

#if IR_USE_STL_STRING
	inline const IrisValue& GetGlobalValue(const string& strValueName, bool& bResult) {
#else
	inline const IrisValue& GetGlobalValue(const IrisInternString& strValueName, bool& bResult) {
#endif // IR_USE_STL_STRING
		bResult = true;
		m_iwlGlobalVariableLock.ReadLock();
		decltype(m_mpGlobalValues)::iterator iGlob;
		if ((iGlob = m_mpGlobalValues.find(strValueName)) == m_mpGlobalValues.end()) {
			bResult = false;
			m_iwlGlobalVariableLock.ReadUnlock();
			return m_ivNil;
		}
		else {
			auto& ivResult = iGlob->second;
			m_iwlGlobalVariableLock.ReadUnlock();
			return ivResult;
		}
	}

#if IR_USE_STL_STRING
	inline const IrisValue& GetOtherValue(const string& strValueName, bool& bResult) {
#else
	inline const IrisValue& GetOtherValue(const IrisInternString& strValueName, bool& bResult) {
#endif // IR_USE_STL_STRING
		bResult = true;
		m_iwlOtherVariableLock.ReadLock();
		decltype(m_mpOtherValues)::iterator iOth;
		if ((iOth = m_mpOtherValues.find(strValueName)) == m_mpOtherValues.end()) {
			bResult = false;
			m_iwlOtherVariableLock.ReadUnlock();
			return m_ivNil;
		}
		else {
			auto& ivResult = iOth->second;
			m_iwlOtherVariableLock.ReadUnlock();
			return ivResult;
		}
	}

#if IR_USE_STL_STRING
	inline IrisMethod* GetMainMethod(const string& strMethodName);
#else
	inline IrisMethod* GetMainMethod(const IrisInternString& strMethodName);
#endif // IR_USE_STL_STRING

#if IR_USE_STL_STRING
	inline void AddMainMethod(const string& strMethodName, IrisMethod* pMethod);
#else
	inline void AddMainMethod(const IrisInternString& strMethodName, IrisMethod* pMethod);
#endif // IR_USE_STL_STRING


	inline const IrisValue& Nil() { return m_ivNil; }
	inline const IrisValue& True() { return m_ivTrue; }
	inline const IrisValue& False() { return m_ivFalse; }

	friend class IrisGC;

	// -------------- Instructors --------------

private:
#if IR_USE_STL_STRING
	bool BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, string& strMethodName);
#else
	bool BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisInternString& strMethodName);
#endif // IR_USE_STL_STRING
	void GetCodesFromBlock(unsigned int nIndex, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisCodeSegment& icsCodeSegment);

	// virtual bytecode
public:
	bool push_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool cre_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool nol_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool assign(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool hid_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool set_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool clr_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool fld_load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load_nil(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load_true(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load_false(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load_self(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool imth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool cmth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool blk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool end_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool jfon(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool jmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool ini_tm(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool ini_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool cmp_tac(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool inc_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool assign_log(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool brk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool rtn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nEnder);
	bool ctn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool assign_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool assign_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool jt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool assign_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool cmp_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool cre_cenv(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool def_cls(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool add_ext(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool add_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool add_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool push_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool pop_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool str_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool gtr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool gstr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool set_auth(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool def_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool def_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool def_infs(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool cblk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool blk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	//bool cast(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool reg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool ureg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool assign_ir(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool grn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool spr(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
	bool load_cast(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);

	friend class IrisKernel;
};

#endif
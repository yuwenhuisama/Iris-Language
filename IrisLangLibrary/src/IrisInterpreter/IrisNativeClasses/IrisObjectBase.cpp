#include "IrisInterpreter/IrisNativeClasses/IrisObjectBase.h"


IrisValue IrisObjectBase::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisObjectBase::GetClass(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IIrisObject* pObject = IrisDevUtil::GetNativeObjectPointer(ivObj);
	return IrisValue::WrapObjectPointerToIrisValue(pObject);
}

IrisValue IrisObjectBase::GetObjectID(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::CreateInt(IrisDevUtil::GetObjectID(ivObj));
}

IrisValue IrisObjectBase::ToString(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	//const string& strClassName = static_cast<IrisObject*>(ivObj.GetIrisObject())->GetClass()->GetInternClass()->GetClassName();
	const string& strClassName = IrisDevUtil::GetNameOfClass(IrisDevUtil::GetClassOfObject(ivObj));
	IrisValue ivStringObjectID = IrisDevUtil::CallMethod(ivObj, "__get_object_id", nullptr);
	const string& strObjectID = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivStringObjectID)->ToString();
	string strOutString = "<" + strClassName + ":" + strObjectID + ">";
	return IrisDevUtil::CreateString(strOutString.c_str());
}

IrisValue IrisObjectBase::Equal(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	const IrisValue& ivDestObj = static_cast<IrisValues*>(ivsValues)->GetValue(0);
	if (ivObj == ivDestObj) {
		return IrisDevUtil::True();
	}
	else {
		return IrisDevUtil::False();
	}
}

IrisValue IrisObjectBase::NotEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivResult = IrisDevUtil::CallMethod(ivObj, "==", ivsValues);
	if (ivResult == IrisDevUtil::True()) {
		return IrisDevUtil::False();
	}
	else {
		return IrisDevUtil::True();
	}
}

IrisValue IrisObjectBase::LogicOr(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue& ivDest = (IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0);
	// 左右两边只要有一个不为nil或false那么就为true
	if (ivObj != IrisDevUtil::False() && ivObj != IrisDevUtil::Nil()) {
		return IrisDevUtil::True();
	}
	else if (ivDest != IrisDevUtil::False() && ivDest != IrisDevUtil::Nil()) {
		return IrisDevUtil::True();
	}
	else {
		return IrisDevUtil::False();
	}
}

IrisValue IrisObjectBase::LogicAnd(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue& ivDest = (IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0);
	// 左右两边只要有一个为nil或false那么就为false
	if (ivObj == IrisDevUtil::False() || ivObj == IrisDevUtil::Nil()) {
		return IrisDevUtil::False();
	}
	//else if (ivDest == IrisDevUtil::False() && ivDest == IrisDevUtil::Nil()) {
	else if (ivDest == IrisDevUtil::False() || ivDest == IrisDevUtil::Nil()) {
		return IrisDevUtil::False();
	}
	else {
		return IrisDevUtil::True();
	}
}

IrisValue IrisObjectBase::Fix(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
	pObject->Fix();
	return IrisDevUtil::Nil();
}

IrisValue IrisObjectBase::Unfix(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
	pObject->Unfix();
	return IrisDevUtil::Nil();
}

IrisValue IrisObjectBase::IsFixed(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
	return pObject->IsFixed() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisObjectBase::IrisObjectBase()
{
}


IrisObjectBase::~IrisObjectBase()
{
}
